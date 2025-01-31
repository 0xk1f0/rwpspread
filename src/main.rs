mod cli;
mod helpers;
mod integrations;
mod watch;
mod wayland;
mod worker;

use cli::Config;
use helpers::Helpers;
use std::panic;
use std::process;
use std::sync::mpsc::sync_channel;
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
            let (tx, rx) = sync_channel(0);

            // watch outputs
            Watcher::output_watch(Wayland::connect()?, tx.clone())?;

            // watch file if desired
            if config.watch {
                Watcher::source_watch(config.raw_input_path.clone(), tx.clone())?;
            }

            loop {
                // rerun if config changed or screens changed
                if let Ok(_) = rx.recv() {
                    // redetect screens
                    let monitors = Wayland::connect()?.get_monitors()?;
                    // rerun splitter
                    if let Some(config) = Config::new()? {
                        worker.run(&config, monitors)?;
                    }
                } else {
                    return Err("watcher threads panicked".to_string());
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
    panic::set_hook(Box::new(|panic| {
        if let Some(s) = panic.payload().downcast_ref::<&str>() {
            eprintln!("{}: panic: \x1B[91m{s:?}\x1B[39m", "rwpspread");
        } else if let Some(s) = panic.payload().downcast_ref::<String>() {
            eprintln!("{}: panic: \x1B[91m{s:?}\x1B[39m", "rwpspread");
        } else {
            eprintln!("{}: panic: \x1B[91m{}\x1B[39m", "rwpspread", "panicked");
        }
        process::exit(1);
    }));

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
