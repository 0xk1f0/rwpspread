mod cli;
mod integrations;
mod wayland;
mod worker;

use cli::Config;
use integrations::helpers;
use std::process;
use wayland::MonitorConfig;
use worker::Worker;

fn run() -> Result<(), String> {
    // create new config
    let worker_config = Config::new().map_err(|err| err)?;

    // check for backends if applicable
    if worker_config.backend.is_some()
        && !helpers::is_installed(&worker_config.backend.as_ref().unwrap().to_string())
    {
        return Err(format!(
            "{} is not installed",
            &worker_config.backend.as_ref().unwrap().to_string()
        ));
    }

    // connect to wayland
    let mut mon_conn = MonitorConfig::new().map_err(|err| err)?;

    // get monitor info
    let mon_config = mon_conn.run().map_err(|err| err)?;

    // create new splitter
    let mut worker = Worker::new();

    // perform split
    worker.run(&worker_config, mon_config).map_err(|err| err)?;

    // check for watchdog bool
    if worker_config.daemon == true {
        loop {
            // roundtrip eventhandler and check result
            let needs_recalc = mon_conn.refresh().map_err(|err| err)?;
            if needs_recalc {
                // redetect screens
                let mon_config = mon_conn.run().map_err(|err| err)?;
                // rerun splitter
                worker.run(&worker_config, mon_config).map_err(|err| err)?;
            }
        }
    }

    // return
    Ok(())
}

fn main() {
    // run with config
    if let Err(err) = run() {
        eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
        process::exit(1);
    }
}
