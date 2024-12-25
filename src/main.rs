mod cli;
mod integrations;
mod wayland;
mod worker;

use cli::Config;
use integrations::helpers;
use std::process;
use wayland::MonitorConfig;
use worker::Worker;

fn run() -> Result<String, String> {
    // create new config
    let worker_config = Config::new()?;

    // connect to wayland
    let mut mon_conn = MonitorConfig::new()?;

    // get monitor info
    let mon_config = mon_conn.run()?;

    // check for backends if applicable
    if let Some(config) = worker_config {
        // check for backends if applicable
        if let Some(run_config) = &config.backend {
            if !helpers::is_installed(&run_config.to_string()) {
                return Err(format!("{} is not installed", &run_config.to_string()));
            }
        }

        // create and execute worker
        let mut worker = Worker::new();
        worker.run(&config, mon_config)?;

        // check for watchdog bool
        if config.daemon == true {
            loop {
                // roundtrip eventhandler and check result
                let needs_recalc = mon_conn.refresh()?;
                if needs_recalc {
                    // redetect screens
                    let mon_config = mon_conn.run()?;
                    // rerun splitter
                    worker.run(&config, mon_config)?;
                }
            }
        }
    } else {
        // since no runtime config was found, return info
        let mut result = String::from("Found the following displays:\n");
        for monitor in mon_config {
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
