use std::env;
use std::process;
use rwpspread::Config;
use colored::Colorize;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args)
        .unwrap_or_else(|err| {
            eprintln!("{}: {}", "rwpspread".red().bold(), err);
            process::exit(1);
        });

    if let Err(err) = rwpspread::run(config) {
        eprintln!("{}: {}", "rwpspread".red().bold(), err);
        process::exit(1);
    }
}
