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
            |err| err.to_string()
        )?;

        // get monitor dimensions
        for monitor in all {
            // always check rotation
            let adj_width: u16;
            let adj_height: u16;
            // we only need reversal on some orientations
            match monitor.transform {
                Transforms::Normal90 | 
                Transforms::Normal270 => {
                    // flipped
                    adj_width = monitor.height;
                    adj_height = monitor.width;
                },
                _ => {
                    // else normal
                    adj_width = monitor.width;
                    adj_height = monitor.height;
                }
            }
            // push to vector
            result.push(
                Monitor {
                    name: monitor.name,
                    width: adj_width as u32,
                    height: adj_height as u32,
                    x: monitor.x,
                    y: monitor.y
                }
            );
        }
        Ok(result)
    }
    // string format for hash calculation
    pub fn to_string(&self) -> String {
        format!(
            "{}{}{}{}{}",
            &self.name,
            &self.x,
            &self.y,
            &self.width,
            &self.height
        )
    }
}
