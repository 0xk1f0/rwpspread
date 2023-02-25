use crate::layout::Monitor;
use clap::Parser;

/// Multi-Monitor Wallpaper Utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
   /// Image File Path
   #[arg(short, long)]
   image: String,

   /// Use wpaperd integration
   #[arg(short, long)]
   wpaperd: bool,
}

pub struct Config {
    pub image_file: String,
    pub mon_list: Vec<Monitor>,
    pub with_wpaperd: bool
}

impl Config {
    pub fn new() -> Self {
        // handle args
        let args = Args::parse();

        /*
            @TODO: 
            -> Make sure we actually are dealing
            with a valid path and image file
            -> Stuff also breaks if the input 
            image is a symlink
        */

        // construct
        Self {
            image_file: args.image,
            mon_list: Monitor::new_from_hyprland().unwrap(),
            with_wpaperd: args.wpaperd
        }
    }
}
