mod palette;
mod parser;
mod splitter;
mod wayland;
mod wpaperd;

use parser::Config;
use splitter::Splitter;
use std::process;
use wayland::MonitorConfig;

fn run() -> Result<(), String> {
    // create new config
    let worker_config = Config::new().map_err(|err| err.to_string())?;

    // connect to wayland
    let mut mon_conn = MonitorConfig::new().map_err(|err| err.to_string())?;

    // get monitor info
    let mon_config = mon_conn.run().map_err(|err| err.to_string())?;

    // create new splitter
    let mut worker = Splitter::new();

    // perform split
    worker
        .run(&worker_config, mon_config)
        .map_err(|err| err.to_string())?;

    // check for watchdog bool
    if worker_config.daemon == true {
        loop {
            // roundtrip eventhandler and check result
            let needs_recalc = mon_conn.refresh().map_err(|err| err.to_string())?;
            if needs_recalc {
                // redetect screens
                let mon_config = mon_conn.run().map_err(|err| err.to_string())?;
                // rerun splitter
                worker
                    .run(&worker_config, mon_config)
                    .map_err(|err| err.to_string())?;
            }
        }
    }

    // return
    Ok(())
}

fn main() {
    // run with config
    if let Err(err) = run() {
        eprintln!("{}: {}", "rwpspread", err);
        process::exit(1);
    }
}
