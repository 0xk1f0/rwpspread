mod parser;
mod splitter;
mod layout;
mod wpaperd;

use std::process;
use std::error::Error;
use std::env::var;
use colored::Colorize;
use parser::Config;
use splitter::split_image;
use wpaperd::WpaperdConfig;

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // perform split operation
    let result = split_image(
        &config
    ).map_err(
       |_| "error while splitting"
    )?;

    // export images
    for paper in &result {
        let _ = paper.image.save(
            &paper.image_full_path
        ).map_err(
            |_| "error while saving"
        )?;
    }

    // check if we have to create a wpaperd config
    if config.with_wpaperd {
        let wpaperd = WpaperdConfig::new(
            format!("{}/.config/wpaperd/output.conf", var("HOME").unwrap()),
            result
        );
        wpaperd.build().map_err(
            |_| "error while creating wpaperd config"
        )?;
    }

    // return
    Ok(())
}

fn main() {
    // create new config
    let config = Config::new();

    // run splitter with config
    if let Err(err) = run(config) {
        eprintln!("{}: {}", "rwpspread".red().bold(), err);
        process::exit(1);
    }
}
