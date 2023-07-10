use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use wayland_client::{
    globals::registry_queue_init, protocol::wl_output, Connection, EventQueue, QueueHandle,
};

struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
    needs_recalc: bool,
}

#[derive(Hash)]
pub struct Monitor {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

pub struct MonitorConfig {
    lo: ListOutputs,
    eq: EventQueue<ListOutputs>,
}

impl MonitorConfig {
    pub fn new() -> Result<Self, String> {
        // Try to connect to the Wayland server.
        let conn = Connection::connect_to_env().map_err(|_| "wayland connection error")?;

        // Now create an event queue and a handle to the queue so we can create objects.
        let (globals, event_queue) =
            registry_queue_init(&conn).map_err(|_| "wayland regqueue error")?;
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
    pub fn run(&mut self) -> Result<Vec<Monitor>, String> {
        // Initialize data
        self.eq
            .roundtrip(&mut self.lo)
            .map_err(|_| "wayland eventqueue error")?;

        // new result vector
        let mut result: Vec<Monitor> = Vec::new();

        // Now our outputs have been initialized with data,
        // we may access what outputs exist and information about
        // said outputs using the output delegate.
        for output in self.lo.output_state.outputs() {
            // get info
            let info = self.lo.output_state.info(&output).unwrap();
            // push to vector
            result.push(Monitor {
                name: info.name.as_ref().unwrap().to_string(),
                width: info.logical_size.unwrap().0 as u32,
                height: info.logical_size.unwrap().1 as u32,
                x: info.logical_position.unwrap().0,
                y: info.logical_position.unwrap().1,
            });
        }

        Ok(result)
    }
    pub fn refresh(&mut self) -> Result<bool, String> {
        // make a roundtrip with queue to dispatch events
        self.eq
            .roundtrip(&mut self.lo)
            .map_err(|_| "wayland eventqueue error")?;

        if self.lo.needs_recalc == true {
            // reset and recalc
            self.lo.needs_recalc = false;
            return Ok(true);
        }

        // nothing to do
        Ok(false)
    }
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

delegate_output!(ListOutputs);
delegate_registry!(ListOutputs);
