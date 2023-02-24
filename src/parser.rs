use crate::hyprland::HyprMonitor;
use clap::Parser;

/// Multi-Monitor Wallpaper Utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
   /// Image File Path
   #[arg(short, long)]
   image: String,
}

pub struct Config {
    pub image_file: String,
    pub mon_list: Vec<HyprMonitor>
}

impl Config {
    pub fn new() -> Result<Config, String> {
        // handle args
        let args = Args::parse();

        /*
            @TODO: Make sure we actually are dealing
            with a valid path and image file
        */

        // clone args to vars
        let image_file: String = args.image;

        // new vector for result imgs
        let mon_list = HyprMonitor::new().map_err(|_| "hyprland failed")?;

        // pass config
        Ok(Config {
            image_file,
            mon_list
        })
    }
}
