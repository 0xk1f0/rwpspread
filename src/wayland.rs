use serde::Serialize;
use smithay_client_toolkit::reexports::client::{
    Connection, EventQueue, QueueHandle, globals::registry_queue_init, protocol::wl_output,
};
use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use std::fmt;

struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
    needs_recalc: bool,
}

impl OutputHandler for ListOutputs {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // set recalc to true
        self.needs_recalc = true
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // set recalc to true
        self.needs_recalc = true
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // set recalc to true
        self.needs_recalc = true
    }
}

impl ProvidesRegistryState for ListOutputs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        OutputState,
    }
}

#[derive(Serialize, Clone)]
pub struct Monitor {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub initial_width: u32,
    pub initial_height: u32,
    pub x: i32,
    pub y: i32,
}

impl fmt::Display for Monitor {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "\x1B[32m{}\x1B[39m: {}x{} at {}:{}",
            self.name, self.width, self.height, self.x, self.y
        )
    }
}

pub struct Wayland {
    lo: ListOutputs,
    eq: EventQueue<ListOutputs>,
}

impl Wayland {
    /// Connect and return a new wayland connection
    pub fn connect() -> Result<Self, String> {
        // Try to connect to the Wayland server.
        let conn = Connection::connect_to_env().map_err(|_| "wayland: failed to connect")?;

        // Now create an event queue and a handle to the queue so we can create objects.
        let (globals, event_queue) =
            registry_queue_init(&conn).map_err(|_| "wayland: failed to init queue")?;
        let qh = event_queue.handle();

        // Initialize the registry handling
        let registry_state = RegistryState::new(&globals);

        // Initialize the delegate we will use for outputs.
        let output_delegate = OutputState::new(&globals, &qh);

        // Set up application state.
        let list_outputs = ListOutputs {
            registry_state,
            output_state: output_delegate,
            needs_recalc: false,
        };

        Ok(Self {
            lo: list_outputs,
            eq: event_queue,
        })
    }
    /// Fetch and return the monitors in the current environment
    pub fn get_monitors(&mut self) -> Result<Vec<Monitor>, String> {
        // Initialize data
        self.eq
            .roundtrip(&mut self.lo)
            .map_err(|_| "wayland: roundtrip failed")?;

        // Now our outputs have been initialized with data,
        // we may access what outputs exist and information about
        // said outputs using the output delegate.
        let mut result: Vec<Monitor> = Vec::with_capacity(self.lo.output_state.outputs().count());
        for output in self.lo.output_state.outputs() {
            // get info
            match self.lo.output_state.info(&output) {
                Some(monitor_info) => {
                    // check for things we need and push
                    result.push(Monitor {
                        name: monitor_info
                            .name
                            .as_ref()
                            .ok_or("wayland: compositor reports no monitor name")?
                            .to_string(),
                        initial_width: monitor_info
                            .logical_size
                            .ok_or("wayland: compositor reports no monitor width")?
                            .0 as u32,
                        initial_height: monitor_info
                            .logical_size
                            .ok_or("wayland: compositor reports no monitor height")?
                            .1 as u32,
                        width: monitor_info
                            .logical_size
                            .ok_or("wayland: compositor reports no monitor width")?
                            .0 as u32,
                        height: monitor_info
                            .logical_size
                            .ok_or("wayland: compositor reports no monitor height")?
                            .1 as u32,
                        x: monitor_info
                            .logical_position
                            .ok_or("wayland: compositor reports no monitor x")?
                            .0,
                        y: monitor_info
                            .logical_position
                            .ok_or("wayland: compositor reports no monitor y")?
                            .1,
                    })
                }
                _ => {
                    return Err("wayland: compositor reports no monitor info".to_string());
                }
            }
        }

        Ok(result)
    }
    /// Refresh and return recalculation boolean of the current wayland connection
    pub fn refresh(&mut self) -> Result<bool, String> {
        // roundtrip
        self.eq
            .roundtrip(&mut self.lo)
            .map_err(|_| "wayland: roundtrip failed")?;
        // flush all in queue
        self.eq.flush().map_err(|_| "wayland: flush failed")?;
        // then wait for event
        self.eq
            .blocking_dispatch(&mut self.lo)
            .map_err(|_| "wayland: event dispatch failed")?;

        Ok(self.lo.needs_recalc)
    }
}

delegate_output!(ListOutputs);
delegate_registry!(ListOutputs);
