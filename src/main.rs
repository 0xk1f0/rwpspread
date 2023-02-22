mod splitter;
use std::env;
use std::process;
use rwpspread::Config;
use colored::Colorize;
use std::error::Error;
use crate::splitter::split_image;

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // perform split operation
    let result = split_image(
        config
    ).map_err(
       |_| "error while splitting"
    )?;
    // export images
    for paper in result {
        let image_name = format!(
            "rwpspread_{}_{}x{}.png",
            paper.name,
            paper.image.width(),
            paper.image.height(),
        );
        paper.image.save(
            image_name
        ).map_err(
            |_| "error while saving"
        )?;
    }
    // return none if success
    Ok(())
}

fn main() {
    // get args from environment
    let args: Vec<String> = env::args().collect();

    // create new config
    let config = Config::new(&args)
        .unwrap_or_else(|err| {
            eprintln!("{}: {}", "rwpspread".red().bold(), err);
            process::exit(1);
        });

    // run splitter with config
    if let Err(err) = run(config) {
        eprintln!("{}: {}", "rwpspread".red().bold(), err);
        process::exit(1);
    }
}
