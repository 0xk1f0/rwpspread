use hyprland::data::{Monitors, Transforms};
use hyprland::prelude::*;

// generic monitor struct
pub struct Monitor {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

impl Monitor {
    pub fn new_from_hyprland() -> Result<Vec<Monitor>, String> {
        // new vector for result imgs
        let mut result: Vec<Monitor> = Vec::new();

        // fetch all
        let all = Monitors::get().map_err(
            |_| ""
        )?;

        // get monitor dimensions
        for monitor in all {
            // always check rotation
            let adj_width: u16;
            let adj_height: u16;
            match monitor.transform {
                Transforms::Normal90 => {
                    adj_width = monitor.height;
                    adj_height = monitor.width;
                },
                Transforms::Normal270 => {
                    adj_width = monitor.height;
                    adj_height = monitor.width;
                },
                _ => {
                    adj_width = monitor.width;
                    adj_height = monitor.height;
                }
            }
            // create new from struct
            let new_mon = Monitor {
                name: monitor.name,
                width: adj_width as u32,
                height: adj_height as u32,
                x: monitor.x,
                y: monitor.y
            };
            // push to vector
            result.push(new_mon);
        }
        Ok(result)
    }
}
