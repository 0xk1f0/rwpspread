mod parser;
mod splitter;
mod wayland;
mod wpaperd;

use colored::Colorize;
use parser::Config;
use splitter::Splitter;
use std::process;

fn run() -> Result<(), String> {
    // create new config
    let worker_config = Config::new().map_err(|err| err.to_string())?;

    // create new splitter
    let worker = Splitter::new();

    // perform split
    worker.run(&worker_config).map_err(|err| err.to_string())?;

    // return
    Ok(())
}

fn main() {
    // run with config
    if let Err(err) = run() {
        eprintln!("{}: {}", "rwpspread".red().bold(), err);
        process::exit(1);
    }
}
