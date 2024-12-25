use serde::Serialize;
use smithay_client_toolkit::reexports::client::{
    globals::registry_queue_init, protocol::wl_output, Connection, EventQueue, QueueHandle,
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

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Clone)]
pub struct Monitor {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

impl Monitor {
    // check monitor collision for specific direction
    pub fn collides_at(&self, direction: &Direction, neighbor: &Monitor) -> bool {
        match direction {
            Direction::Up => {
                if (self.y == neighbor.y + neighbor.height as i32)
                    && (self.x >= neighbor.x && self.x <= neighbor.x + neighbor.width as i32
                        || self.x + self.width as i32 >= neighbor.x
                            && self.x <= neighbor.x + neighbor.width as i32)
                {
                    return true;
                }
            }
            Direction::Down => {
                if (self.y + self.height as i32 == neighbor.x)
                    && (self.x >= neighbor.x && self.x <= neighbor.x + neighbor.width as i32
                        || self.x + self.width as i32 >= neighbor.x
                            && self.x <= neighbor.x + neighbor.width as i32)
                {
                    return true;
                }
            }
            Direction::Left => {
                if (self.x == neighbor.x + neighbor.width as i32)
                    && (self.y >= neighbor.y && self.y <= neighbor.y + neighbor.height as i32
                        || self.y + self.height as i32 >= neighbor.y
                            && self.y <= neighbor.y + neighbor.height as i32)
                {
                    return true;
                }
            }
            Direction::Right => {
                if (self.x + self.width as i32 == neighbor.x)
                    && (self.y >= neighbor.y && self.y <= neighbor.y + neighbor.height as i32
                        || self.y + self.height as i32 >= neighbor.y
                            && self.y <= neighbor.y + neighbor.height as i32)
                {
                    return true;
                }
            }
        }

        false
    }
    // check monitor collision for all available directions
    pub fn collides(&self, neighbor: &Monitor) -> Option<&Direction> {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .iter()
        .find(|&possible_direction| {
            if self.collides_at(possible_direction, neighbor) {
                true
            } else {
                false
            }
        })
    }
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
            if let Some(monitor_info) = self.lo.output_state.info(&output) {
                // check for things we need and push
                result.push(Monitor {
                    name: monitor_info
                        .name
                        .as_ref()
                        .ok_or("wayland: compositor reports no monitor names")?
                        .to_string(),
                    width: monitor_info
                        .logical_size
                        .ok_or("wayland: compositor reports no monitor size")?
                        .0 as u32,
                    height: monitor_info
                        .logical_size
                        .ok_or("wayland: compositor reports no monitor size")?
                        .1 as u32,
                    x: monitor_info
                        .logical_position
                        .ok_or("wayland: compositor reports no monitor position")?
                        .0,
                    y: monitor_info
                        .logical_position
                        .ok_or("wayland: compositor reports no monitor position")?
                        .1,
                });
            } else {
                return Err("wayland: compositor reports no monitor info".to_string());
            }
        }

        Ok(result)
    }

    pub fn refresh(&mut self) -> Result<bool, String> {
        // dispatch events
        self.eq
            .blocking_dispatch(&mut self.lo)
            .map_err(|_| "wayland eventqueue error")?;

        // check if recalculation boolean was set
        if self.lo.needs_recalc == true {
            // reset and recalc
            self.lo.needs_recalc = false;
            return Ok(true);
        }

        Ok(false)
    }
}

delegate_output!(ListOutputs);
delegate_registry!(ListOutputs);
