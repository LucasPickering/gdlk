use crate::{
    error::ServerError,
    models::{QueryHardwareSpec, QueryProgramSpec},
    schema::{hardware_specs::dsl::*, program_specs::dsl::*},
};
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use diesel::{prelude::*, r2d2::ConnectionManager, PgConnection};
use gdlk::{
    compile, CompileErrors, HardwareSpec, Machine, ProgramSpec, RuntimeError,
};
use serde::{Deserialize, Serialize};
use std::{
    convert,
    time::{Duration, Instant},
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// All the different types of events that we can receive over the websocket.
/// These events are typically triggered by user input, but might not
/// necessarily be.
#[derive(Debug, Deserialize)]
#[serde(
    tag = "type",
    content = "content",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum IncomingEvent {
    Edit {
        // Saving room for more fields here
        source: String,
    },
    Compile,
    Step,
}

/// All the different types of events that we can transmit over the websocket.
/// This can include both success and error events.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
enum OutgoingEvent<'a> {
    // OK events
    /// Send latest version of the program source code
    Source {
        // Saving room for more fields here
        source: &'a str,
    },
    /// Send latest version of the machine state
    MachineState {
        state: &'a Machine,
        is_complete: bool,
        is_successful: bool,
    },

    // Error events
    /// Failed to parse websocket message
    MalformedMessage(String),
    /// Failed to parse the sent program
    CompileError(CompileErrors),
    /// Error occurred while running a program
    RuntimeError(RuntimeError),
    /// "Step" message occurred before "Compile" message
    NoCompilation,
}

// Define type conversions to make processing code a bit cleaner

impl<'a> From<&'a Machine> for OutgoingEvent<'a> {
    fn from(machine: &'a Machine) -> Self {
        OutgoingEvent::MachineState {
            state: machine,
            is_complete: machine.is_complete(),
            is_successful: machine.is_successful(),
        }
    }
}

impl<'a> From<serde_json::Error> for OutgoingEvent<'a> {
    fn from(other: serde_json::Error) -> Self {
        OutgoingEvent::MalformedMessage(format!("{}", other))
    }
}

impl<'a> From<CompileErrors> for OutgoingEvent<'a> {
    fn from(errors: CompileErrors) -> Self {
        OutgoingEvent::CompileError(errors)
    }
}

impl<'a> From<RuntimeError> for OutgoingEvent<'a> {
    fn from(error: RuntimeError) -> Self {
        OutgoingEvent::RuntimeError(error)
    }
}

/// The controlling struct for a single websocket instance
struct ProgramWebsocket {
    /// "Hardware" to build/execute the program under, pulled from the DB
    hardware_spec: HardwareSpec,
    /// Specs for the program execution
    program_spec: ProgramSpec,
    /// Track the last time we pinged/ponged the client, if this exceeds
    /// CLIENT_TIMEOUT, drop the connection
    heartbeat: Instant,
    /// The current user-entered program source code
    source_code: String,
    /// The current execution state of the machine. None if the program hasn't
    /// been compiled yet.
    machine: Option<Machine>,
}

impl ProgramWebsocket {
    fn new(hardware_spec: HardwareSpec, program_spec: ProgramSpec) -> Self {
        ProgramWebsocket {
            hardware_spec,
            program_spec,
            heartbeat: Instant::now(),
            source_code: String::new(),
            machine: None,
        }
    }

    /// Processes the given text message, and returns the appropriate response
    /// event. The return type on this is a little funky because all our
    /// event types (OK and error) are under the same enum. We still use a
    /// Result because it makes it easier to exit early in the case of an error.
    fn process_msg(
        &mut self,
        text: String,
    ) -> Result<OutgoingEvent, OutgoingEvent> {
        // Parse the message
        let socket_msg = serde_json::from_str::<IncomingEvent>(&text)?;

        // Process message based on type
        Ok(match socket_msg {
            IncomingEvent::Edit { source } => {
                // Update source code
                self.source_code = source;
                // source code has changed, machine is no longer valid
                self.machine = None;
                OutgoingEvent::Source {
                    source: &self.source_code,
                }
            }
            IncomingEvent::Compile => {
                // Compile the program into a machine
                self.machine = Some(compile(
                    &self.hardware_spec,
                    &self.program_spec,
                    self.source_code.clone(),
                )?);

                // we need this fuckery cause lol borrow checker
                self.machine.as_ref().unwrap().into()
            }
            IncomingEvent::Step => {
                // Execute one step on the machine
                if let Some(machine) = self.machine.as_mut() {
                    machine.execute_next()?;
                    (&*machine).into() // need to convert &mut to just &
                } else {
                    return Err(OutgoingEvent::NoCompilation);
                }
            }
        })
    }
}

impl Actor for ProgramWebsocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. Kick off an interval that pings the
    /// client periodically and also checks if they have timed out.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check if client has timed out
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                // Timed out, kill the connection
                ctx.stop();
            } else {
                // Not timed out, send another ping
                ctx.ping("");
            }
        });
    }
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for ProgramWebsocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            ws::Message::Ping(msg) => {
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.heartbeat = Instant::now();
            }
            ws::Message::Text(text) => {
                // This is a little funky because both sides of the Result are
                // the same type
                let response =
                    self.process_msg(text).unwrap_or_else(convert::identity);
                let response_string = serde_json::to_string(&response).unwrap();

                ctx.text(response_string);
            }
            ws::Message::Binary(_) => {}
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => {}
        }
    }
}

/// Do websocket handshake, look up the request ProgramSpec by ID, then (if it
/// exists), start a handler for it.
pub fn ws_program_specs_by_id(
    r: HttpRequest,
    pool: web::Data<Pool>,
    program_spec_id: web::Path<i32>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let conn: &PgConnection = &pool.get().unwrap();
    // Look up the program spec by ID, get the associated hardware spec too
    let (program_spec, hardware_spec): (QueryProgramSpec, QueryHardwareSpec) =
        program_specs
            .find(program_spec_id.into_inner())
            .inner_join(hardware_specs)
            .get_result(conn)
            .map_err(ServerError::from)?;
    ws::start(
        ProgramWebsocket::new(hardware_spec.into(), program_spec.into()),
        &r,
        stream,
    )
}
