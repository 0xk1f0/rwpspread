mod parser;
mod splitter;
mod layout;
mod wpaperd;

use std::process;
use colored::Colorize;
use parser::Config;
use splitter::Splitter;

fn run() -> Result<(), String> {
    // create new config
    let worker_config = Config::new().map_err(
        |err| err.to_string()
    )?;

    // create new splitter
    let worker = Splitter::new();

    // perform split
    worker.run(&worker_config).map_err(
        |err| err.to_string(),
    )?;

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
