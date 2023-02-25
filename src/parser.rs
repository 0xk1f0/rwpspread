use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;
use crate::layout::Monitor;

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
    pub image_path: PathBuf,
    pub mon_list: Vec<Monitor>,
    pub with_wpaperd: bool
}

impl Config {
    pub fn new() -> Self {
        // handle args
        let args = Args::parse();

        // create new path for image
        let in_path = check_path(Path::new(&args.image));

        // construct
        Self {
            image_path: in_path,
            mon_list: Monitor::new_from_hyprland().unwrap(),
            with_wpaperd: args.wpaperd
        }
    }
}

fn is_symlink(path: &Path) -> bool {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        metadata.file_type().is_symlink()
    } else {
        false
    }
}

fn check_path(path: &Path) -> PathBuf {
    if is_symlink(path) {
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let target = fs::read_link(path).unwrap();
        parent.join(target)
    } else {
        path.to_path_buf()
    }
}
