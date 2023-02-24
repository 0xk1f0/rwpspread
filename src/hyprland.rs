use hyprland::data::{Monitors, Transforms};
use hyprland::prelude::*;

pub struct HyprMonitor {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

impl HyprMonitor {
    pub fn new() -> Result<Vec<HyprMonitor>, String> {
        // new vector for result imgs
        let mut result: Vec<HyprMonitor> = Vec::new();

        // fetch all
        let all = Monitors::get().map_err(
            |_| ""
        )?;

        // get monitor dimensions
        for monitor in all {
            // always check rotation
            let mut adj_width = monitor.width;
            let mut adj_height = monitor.height;
            match monitor.transform {
                Transforms::Normal90 => {
                    adj_width = monitor.height;
                    adj_height = monitor.width;
                },
                Transforms::Normal270 => {
                    adj_width = monitor.height;
                    adj_height = monitor.width;
                },
                _ => {}
            }
            // create new from struct
            let new_mon = HyprMonitor {
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
