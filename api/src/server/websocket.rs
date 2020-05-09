use crate::{
    error::ResponseError,
    models,
    schema::{hardware_specs, program_specs},
    server::Pool,
};
use actix::{Actor, ActorContext, AsyncContext, SpawnHandle, StreamHandler};
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
    /// Initiate a compilation of the given source code. If successful, the
    /// resulting [Machine] will be stored for execution.
    #[serde(rename_all = "camelCase")]
    Compile {
        /// The code to compile
        source_code: String,
    },
    /// Execute one step in the stored machine. Returns an
    /// [OutgoingEvent::NoCompilation] if there is no machine to execute.
    Step,
    /// Enable auto-step, which will automatically step through the the program
    /// at the specified interval.
    #[serde(rename_all = "camelCase")]
    AutoStepStart {
        /// The time, in milliseconds, between steps
        interval: u64,
    },
    /// Disable auto-step.
    AutoStepStop,
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
        cycle_count: usize,
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

impl OutgoingEvent<'_> {
    /// Send this event out over the websocket.
    fn send(&self, ctx: &mut <ProgramWebsocket as Actor>::Context) {
        ctx.text(serde_json::to_string(self).unwrap());
    }
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
            cycle_count: machine.cycle_count(),
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
    /// The duration between steps when auto-step is enabled.
    auto_step_interval: Duration,
    /// The identifier for the auto-step future. This will be populated
    /// iff the auto-stepper is running, and can be used to stop it.
    auto_step_handle: Option<SpawnHandle>,
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
            auto_step_interval: Duration::default(),
            auto_step_handle: None,
        }
    }

    /// Take one step on the current machine.
    fn machine_step(&mut self) -> Result<OutgoingEvent, OutgoingEvent> {
        match self.machine.as_mut() {
            None => Err(OutgoingEvent::NoCompilation),
            Some(machine) => {
                machine.execute_next()?;
                // need to convert &mut to just &
                Ok((machine as &Machine).into())
            }
        }
    }

    /// Start the auto-step process, which will automatically advance the
    /// program on a set interval, until it errors or terminates.
    fn start_auto_step(&mut self, ctx: &mut <Self as Actor>::Context) {
        // Cancel the stepper if it's already running. If it's not, this will do
        // nothing.
        self.cancel_auto_step(ctx);
        self.auto_step_handle =
            Some(ctx.run_interval(self.auto_step_interval, |act, ctx| {
                let response_result = act.machine_step();

                // We need to check this first, before we move the response
                // event out of the `response_result` value
                let should_cancel: bool = match response_result {
                    Err(_) => true,
                    Ok(OutgoingEvent::MachineState { is_complete, .. }) => {
                        is_complete
                    }
                    // This shouldn't happen because MachineState is the only
                    // valid success response
                    Ok(_) => unreachable!(),
                };

                let response: OutgoingEvent =
                    response_result.unwrap_or_else(convert::identity);
                response.send(ctx);
                // Now that `response` is dropped, we can borrow `act` again

                // The actual cancel has to happen _after_ sending the message,
                // because we can only maintain one borrow of `act` at a time
                // (because it's mutable), and `response` has a borrow to it
                if should_cancel {
                    act.cancel_auto_step(ctx);
                }
            }));
    }

    /// Cancel the auto-step process. If it isn't running, this does nothing.
    fn cancel_auto_step(&mut self, ctx: &mut <Self as Actor>::Context) {
        match self.auto_step_handle {
            None => {}
            Some(handle) => {
                ctx.cancel_future(handle);
            }
        }
    }

    /// Processes the given text message, and returns the appropriate response
    /// event. The return type on this is a little funky because all our
    /// event types (OK and error) are under the same enum. We still use a
    /// Result because it makes it easier to exit early in the case of an error.
    /// Error cases always emit a response, but success cases don't necessarily.
    fn process_msg(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        text: String,
    ) -> Result<Option<OutgoingEvent>, OutgoingEvent> {
        // Parse the message
        let socket_msg = serde_json::from_str::<IncomingEvent>(&text)?;

        // Process message based on type
        let response: Option<OutgoingEvent> = match socket_msg {
            IncomingEvent::Compile { source_code } => {
                // Compile the program into a machine
                self.machine = Some(
                    Compiler::compile(source_code, self.hardware_spec)?
                        .allocate(&self.program_spec),
                );

                // we need this fuckery cause lol borrow checker
                Some(self.machine.as_ref().unwrap().into())
            }
            IncomingEvent::Step => Some(self.machine_step()?),
            IncomingEvent::AutoStepStart { interval } => {
                self.auto_step_interval = Duration::from_millis(interval);
                self.start_auto_step(ctx);
                None
            }
            IncomingEvent::AutoStepStop => {
                // Stop the auto-stepper
                self.cancel_auto_step(ctx);
                None
            }
        };
        Ok(response)
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
                match self.process_msg(ctx, text) {
                    Ok(None) => {}
                    // If a response was given, send it over the wire
                    Ok(Some(response)) | Err(response) => {
                        response.send(ctx);
                    }
                }
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
