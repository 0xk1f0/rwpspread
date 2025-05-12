use crate::helpers::Helpers;
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
use std::collections::HashMap;
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

#[derive(PartialEq)]
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
    pub initial_width: u32,
    pub initial_height: u32,
    pub x: i32,
    pub y: i32,
}

impl Monitor {
    /// Check for monitor neighbor collision in specific direction
    pub fn collides_with_at(&self, neighbor: &Monitor, direction: &Direction) -> bool {
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
                if (self.y + self.height as i32 == neighbor.y)
                    && ((self.x >= neighbor.x || self.x + self.width as i32 >= neighbor.x)
                        && self.x <= neighbor.x + neighbor.width as i32)
                {
                    return true;
                }
            }
            Direction::Left => {
                if (self.x == neighbor.x + neighbor.width as i32)
                    && ((self.y >= neighbor.y || self.y + self.height as i32 >= neighbor.y)
                        && self.y <= neighbor.y + neighbor.height as i32)
                {
                    return true;
                }
            }
            Direction::Right => {
                if (self.x + self.width as i32 == neighbor.x)
                    && ((self.y >= neighbor.y || self.y + self.height as i32 >= neighbor.y)
                        && self.y <= neighbor.y + neighbor.height as i32)
                {
                    return true;
                }
            }
        }

        false
    }
    /// Check for monitor neighbor collision in any direction
    pub fn collides_with(&self, neighbor: &Monitor) -> Option<&Direction> {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .iter()
        .find(|&possible_direction| {
            if self.collides_with_at(neighbor, possible_direction) {
                true
            } else {
                false
            }
        })
    }
    /// Calculate and return ppi based on monitor diagonal in inches
    pub fn ppi(&self, diagonal_inches: u32) -> u32 {
        let diagonal_pixels = ((self.width).pow(2) + (self.height).pow(2)).isqrt() as u64;

        (diagonal_pixels as f64 / (diagonal_inches as f64)).round() as u32
    }
    /// Calculate and return the absolute shift amount based on the difference of the scaled and inital width
    pub fn abs_shift_diff(&self) -> (i32, i32) {
        let x_diff: i32 = self.initial_width as i32 - self.width as i32;
        let y_diff: i32 = self.initial_height as i32 - self.height as i32;

        ((x_diff / 2).abs(), (y_diff / 2).abs())
    }
    /// Scale and save the new monitor size based on scale factor
    pub fn scale(&mut self, scale_factor: f32) -> &mut Self {
        self.width = Helpers::round_2((self.width as f32 * scale_factor).round() as u32);
        self.height = Helpers::round_2((self.height as f32 * scale_factor).round() as u32);

        self
    }
    /// Shift and save a new x and y position of monitor
    pub fn shift(&mut self, x_amount: i32, y_amount: i32) -> &mut Self {
        self.x += x_amount;
        self.y += y_amount;

        self
    }
    /// Center and save new centered x and y position monitor based on shift amount
    pub fn center(&mut self) -> &mut Self {
        let (x_shift, y_shift) = self.abs_shift_diff();
        self.x += x_shift;
        self.y += y_shift;

        self
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
    pub fn get_monitors(&mut self) -> Result<HashMap<String, Monitor>, String> {
        // Initialize data
        self.eq
            .roundtrip(&mut self.lo)
            .map_err(|_| "wayland: roundtrip failed")?;

        // Now our outputs have been initialized with data,
        // we may access what outputs exist and information about
        // said outputs using the output delegate.
        let mut result: HashMap<String, Monitor> =
            HashMap::with_capacity(self.lo.output_state.outputs().count());
        for output in self.lo.output_state.outputs() {
            // get info
            match self.lo.output_state.info(&output) {
                Some(monitor_info) => {
                    // check for things we need and push
                    result.insert(
                        monitor_info
                            .name
                            .as_ref()
                            .ok_or("wayland: compositor reports no monitor name")?
                            .to_string(),
                        Monitor {
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
                        },
                    );
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
