mod splitter;
mod parser;
mod hyprland;
use std::process;
use std::error::Error;
use colored::Colorize;
use parser::Config;
use splitter::split_image;

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
    // create new config
    let config = Config::new()
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
