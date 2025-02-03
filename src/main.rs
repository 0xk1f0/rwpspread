mod cli;
mod helpers;
mod integrations;
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
    // create new config
    let worker_config = Config::new()?;

    // get monitors
    let monitors = Wayland::connect()?.get_monitors()?;

    // check for backends if applicable
    if let Some(config) = worker_config {
        // check for backends if applicable
        if let Some(run_config) = &config.backend {
            if !Helpers::is_installed(&run_config.to_string()) {
                return Err(format!("{} is not installed", &run_config.to_string()));
            }
        }

        // create and execute worker
        let mut worker = Worker::new();
        worker.run(&config, monitors)?;

        // check for watchdog bool
        if config.daemon {
            let (tx_monitors, rx_monitors) = bounded::<&str>(1);
            let (tx_file, rx_file) = bounded::<&str>(1);

            // watch outputs
            Watcher::monitors(Wayland::connect()?, tx_monitors.clone())?;

            // watch file if desired
            if config.watch {
                Watcher::file(config.raw_input_path.clone(), tx_file.clone())?;
            }

            loop {
                select! {
                    recv(rx_monitors) -> _ => {
                        if let Some(config) = Config::new()? {
                            worker.run(&config, Wayland::connect()?.get_monitors()?)?;
                        }
                        // restart thread
                        Watcher::monitors(Wayland::connect()?, tx_monitors.clone())?;
                    }
                    recv(rx_file) -> _ => {
                        if let Some(config) = Config::new()? {
                            worker.run(&config, Wayland::connect()?.get_monitors()?)?;
                        }
                        // restart file watch thread
                        Watcher::file(config.raw_input_path.clone(), tx_file.clone())?;
                    }
                }
            }
        }
    } else {
        // since no runtime config was found, return info
        let mut result = String::from("Found the following displays:\n");
        for monitor in monitors {
            result.push_str(&format!("- {}\n", monitor));
        }
        result.push_str("Supported versions:\n");
        result.push_str(&format!(
            "- \x1B[32mhyprpaper\x1B[39m: {}\n- \x1B[32mwpaperd\x1B[39m: {}\n- \x1B[32mswaybg\x1B[39m: {}",
            env!("HYPRPAPER_VERSION"),
            env!("WPAPERD_VERSION"),
            env!("SWAYBG_VERSION")
        ));
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
