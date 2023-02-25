mod parser;
mod splitter;
mod layout;
mod wpaperd;

use std::process;
use colored::Colorize;
use parser::Config;
use splitter::Splitter;

fn run(config: Config) -> Result<(), String> {
    // create new splitter
    let worker = Splitter::new(config);

    // perform split
    worker.run().map_err(
       |_| "error while splitting"
    )?;

    // return
    Ok(())
}

fn main() {
    // create new config
    if let Err(err) = Config::new() {
        eprintln!("{}: {}", "rwpspread".red().bold(), err);
        process::exit(1);
    }

    // run with config
    if let Err(err) = run(Config::new().unwrap()) {
        eprintln!("{}: {}", "rwpspread".red().bold(), err);
        process::exit(1);
    }
}
