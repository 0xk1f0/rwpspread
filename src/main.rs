mod cli;
mod helpers;
mod integrations;
mod layout;
mod watch;
mod wayland;
mod worker;

use cli::Config;
use crossbeam_channel::{bounded, select};
use helpers::Helpers;
use std::process;
use watch::Watcher;
use wayland::Wayland;
use worker::Worker;

fn run() -> Result<String, String> {
    // check for backends if applicable
    if let Some(config) = Config::new()? {
        // check for backends if applicable
        if let Some(backend) = &config.backend {
            if !Helpers::is_installed(&backend.to_string()) {
                return Err(format!("{} is not installed", &backend.to_string()));
            }
        }

        if config.daemon {
            // run worker initially
            Worker::new().run(&config, Wayland::connect()?.get_monitors()?)?;

            if config.watch {
                // create two channels for threads
                let (tx_monitors, rx_monitors) = bounded::<bool>(1);
                let (tx_file, rx_file) = bounded::<bool>(1);

                // start both operations seperately
                let mut monitors_handle =
                    Watcher::monitors(Wayland::connect()?, tx_monitors.clone())?;
                let mut file_handle =
                    Watcher::file(config.raw_input_path.clone(), tx_file.clone())?;

                loop {
                    // watch for thread channel events, resplit and restart thread
                    select! {
                        recv(rx_monitors) -> _ => {
                            monitors_handle.join().map_err(|_| "thread: rwp_monitors panicked")?;
                            if let Some(config) = Config::new()? {
                                Worker::new().run(&config, Wayland::connect()?.get_monitors()?)?;
                            }
                            monitors_handle = Watcher::monitors(Wayland::connect()?, tx_monitors.clone())?;
                        }
                        recv(rx_file) -> _ => {
                            file_handle.join().map_err(|_| "thread: rwp_file panicked")?;
                            if let Some(config) = Config::new()? {
                                Worker::new().run(&config, Wayland::connect()?.get_monitors()?)?;
                            }
                            file_handle = Watcher::file(config.raw_input_path.clone(), tx_file.clone())?;
                        }
                    }
                }
            } else {
                let (tx_monitors, rx_monitors) = bounded::<bool>(1);

                let mut monitors_handle =
                    Watcher::monitors(Wayland::connect()?, tx_monitors.clone())?;

                loop {
                    if let Ok(_) = rx_monitors.recv() {
                        monitors_handle
                            .join()
                            .map_err(|_| "thread: rwp_monitors panicked")?;
                        if let Some(config) = Config::new()? {
                            Worker::new().run(&config, Wayland::connect()?.get_monitors()?)?;
                        }
                        monitors_handle =
                            Watcher::monitors(Wayland::connect()?, tx_monitors.clone())?;
                    }
                }
            }
        } else {
            // run worker once
            Worker::new().run(&config, Wayland::connect()?.get_monitors()?)?;
        }
    } else {
        // since no runtime config was found, return info
        let mut result = String::new();
        let monitors = Wayland::connect()?.get_monitors()?;
        for (x, (name, mon)) in monitors.iter().enumerate() {
            result.push_str(&format!(
                "\x1B[1m\x1B[4m{}:\x1B[0m \x1B[3m{}x{}\x1B[0m at \x1B[3m{}:{}\x1B[0m",
                name, mon.width, mon.height, mon.x, mon.y
            ));
            if x + 1 != monitors.len() {
                result.push_str("\n");
            }
        }
        return Ok(result);
    }

    Ok("".to_string())
}

fn main() {
    match run() {
        Ok(ok) => {
            println!("{}", ok);
            process::exit(0);
        }
        Err(err) => {
            eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
            process::exit(1);
        }
    }
}
