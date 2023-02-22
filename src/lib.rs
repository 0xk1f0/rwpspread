use hyprland::data::{Monitors, Transforms};
use hyprland::prelude::*;

// generic monitor struct
pub struct HyprMonitor {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

pub struct Config {
    pub image_file: String,
    pub mon_list: Vec<HyprMonitor>
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &str> {
        // handle args
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 2 {
            return Err("too many arguments");
        }

        /*
            @TODO: Make sure we actually are dealing
            with a valid path and image file
        */
        // clone args to vars
        let image_file: String = args[1].clone();

        // new vector for result imgs
        let mut mon_list: Vec<HyprMonitor> = Vec::new();

        // fetch all
        let all = Monitors::get().unwrap();

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
            mon_list.push(new_mon);
        }

        // pass config
        Ok(Config {
            image_file,
            mon_list
        })
    }
}
