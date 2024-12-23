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
    if worker_config.is_none() {
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
    let ready_config = worker_config.unwrap();

    // check for backends if applicable
    if ready_config.backend.is_some()
        && !helpers::is_installed(&ready_config.backend.as_ref().unwrap().to_string())
    {
        return Err(format!(
            "{} is not installed",
            &ready_config.backend.as_ref().unwrap().to_string()
        ));
    }

    // create new splitter
    let mut worker = Worker::new();

    // perform split
    worker.run(&ready_config, mon_config)?;

    // check for watchdog bool
    if ready_config.daemon == true {
        loop {
            // roundtrip eventhandler and check result
            let needs_recalc = mon_conn.refresh()?;
            if needs_recalc {
                // redetect screens
                let mon_config = mon_conn.run()?;
                // rerun splitter
                worker.run(&ready_config, mon_config)?;
            }
        }
    }

    // return
    Ok("".to_string())
}

fn main() {
    // run with config
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
