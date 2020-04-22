use crate::{
    error::ResponseError,
    models,
    schema::{hardware_specs, program_specs},
    server::Pool,
};
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{get, web, HttpRequest, HttpResponse, ResponseError as _};
use actix_web_actors::ws;
use diesel::{prelude::*, PgConnection};
use gdlk::{
    ast::{LangValue, RegisterRef, StackRef},
    error::{CompileError, RuntimeError, WithSource},
    validator::ValidationErrors,
    Compiler, HardwareSpec, Machine, ProgramSpec, Valid,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert,
    convert::TryInto,
    time::{Duration, Instant},
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// All the different types of events that we can receive over the websocket.
/// These events are typically triggered by user input, but might not
/// necessarily be.
#[derive(Debug, Deserialize)]
#[serde(
    tag = "type",
    content = "content",
    rename_all = "camelCase",
    deny_unknown_fields
)]
enum IncomingEvent {
    #[serde(rename_all = "camelCase")]
    Compile {
        // Saving room for more fields here
        source_code: String,
    },
    Step,
}

/// All the different types of events that we can transmit over the websocket.
/// This can include both success and error events.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
enum OutgoingEvent<'a> {
    // OK events
    /// Send latest version of the machine state
    #[serde(rename_all = "camelCase")]
    MachineState {
        program_counter: usize,
        input: &'a [LangValue],
        output: &'a [LangValue],
        registers: HashMap<RegisterRef, LangValue>,
        stacks: HashMap<StackRef, &'a [LangValue]>,
        is_complete: bool,
        is_successful: bool,
    },

    // Error events
    /// Failed to parse websocket message
    MalformedMessage(String),
    /// Failed to compile the sent program
    CompileError(WithSource<CompileError>),
    /// Error occurred while running a program
    RuntimeError(WithSource<RuntimeError>),
    /// "Step" message occurred before "Compile" message
    NoCompilation,
}

// Define type conversions to make processing code a bit cleaner

impl<'a> From<&'a Machine> for OutgoingEvent<'a> {
    fn from(machine: &'a Machine) -> Self {
        OutgoingEvent::MachineState {
            program_counter: machine.program_counter(),
            input: machine.input(),
            output: machine.output(),
            registers: machine.registers(),
            stacks: machine.stacks(),
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

impl<'a> From<ValidationErrors> for OutgoingEvent<'a> {
    fn from(other: ValidationErrors) -> Self {
        OutgoingEvent::MalformedMessage(format!("{}", other))
    }
}

impl<'a> From<WithSource<CompileError>> for OutgoingEvent<'a> {
    fn from(errors: WithSource<CompileError>) -> Self {
        OutgoingEvent::CompileError(errors)
    }
}

impl<'a> From<WithSource<RuntimeError>> for OutgoingEvent<'a> {
    fn from(error: WithSource<RuntimeError>) -> Self {
        OutgoingEvent::RuntimeError(error)
    }
}

/// The controlling struct for a single websocket instance
struct ProgramWebsocket {
    /// "Hardware" to build/execute the program under, pulled from the DB
    hardware_spec: Valid<HardwareSpec>,
    /// Specs for the program execution
    program_spec: Valid<ProgramSpec>,
    /// Track the last time we pinged/ponged the client, if this exceeds
    /// CLIENT_TIMEOUT, drop the connection
    heartbeat: Instant,
    /// The current execution state of the machine. None if the program hasn't
    /// been compiled yet.
    machine: Option<Machine>,
}

impl ProgramWebsocket {
    fn new(
        hardware_spec: Valid<HardwareSpec>,
        program_spec: Valid<ProgramSpec>,
    ) -> Self {
        ProgramWebsocket {
            hardware_spec,
            program_spec,
            heartbeat: Instant::now(),
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
            IncomingEvent::Compile { source_code } => {
                // Compile the program into a machine
                self.machine = Some(
                    Compiler::compile(source_code, self.hardware_spec)?
                        .allocate(&self.program_spec),
                );

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
                ctx.ping(b"");
            }
        });
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for ProgramWebsocket
{
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // This is a little funky because both sides of the Result are
                // the same type
                let response =
                    self.process_msg(text).unwrap_or_else(convert::identity);
                let response_string = serde_json::to_string(&response).unwrap();

                ctx.text(response_string);
            }
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }

            // Don't do anything with these messages
            Ok(ws::Message::Binary(_))
            | Ok(ws::Message::Continuation(_))
            | Ok(ws::Message::Nop)
            | Err(_) => {}
        }
    }
}

/// Do websocket handshake, look up the request ProgramSpec by ID, then (if it
/// exists), start a handler for it.
#[get("/ws/hardware/{hw_spec_slug}/programs/{program_spec_slug}")]
pub async fn ws_program_specs_by_slugs(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<Pool>,
    params: web::Path<(String, String)>,
) -> Result<HttpResponse, actix_web::Error> {
    let conn = &pool.get().map_err(ResponseError::from)? as &PgConnection;
    let (hw_spec_slug, program_spec_slug) = params.into_inner();
    // Look up the program spec by ID, get the associated hardware spec too
    let (program_spec, hardware_spec): (
        models::ProgramSpec,
        models::HardwareSpec,
    ) = match program_specs::table
        .inner_join(hardware_specs::table)
        .filter(hardware_specs::dsl::slug.eq(&hw_spec_slug))
        .filter(program_specs::dsl::slug.eq(&program_spec_slug))
        .get_result::<(models::ProgramSpec, models::HardwareSpec)>(conn)
        .optional()
    {
        Ok(Some(rows)) => Ok(rows),
        // Convert missing row to 404
        Ok(None) => Err(HttpResponse::NotFound().into()),
        Err(err) => Err(ResponseError::from(err).error_response()),
    }?;
    ws::start(
        ProgramWebsocket::new(
            // These unwraps _should_ be safe because our DB constraints
            // and input validation prevent validation errors here
            hardware_spec.try_into().unwrap(),
            program_spec.try_into().unwrap(),
        ),
        &req,
        stream,
    )
}
