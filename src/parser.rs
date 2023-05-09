use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

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

    /// Force Resplit even if cache exists
    #[arg(short, long)]
    force_resplit: bool,

    /// Don't downscale base image, even if it's bigger than needed
    #[arg(short, long)]
    dont_downscale: bool,
}

pub struct Config {
    pub image_path: PathBuf,
    pub with_wpaperd: bool,
    pub force_resplit: bool,
    pub dont_downscale: bool,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        // handle args
        let args = Args::parse();

        // check if path is valid
        if !fs::metadata(Path::new(&args.image)).is_ok() {
            Err("Invalid Path")?
        }

        // create new path for image
        let in_path = Config::check_path(Path::new(&args.image));

        // construct
        Ok(Self {
            image_path: in_path,
            with_wpaperd: args.wpaperd,
            force_resplit: args.force_resplit,
            dont_downscale: args.dont_downscale,
        })
    }

    // check if target path is a symlink
    fn is_symlink(path: &Path) -> bool {
        if let Ok(metadata) = fs::symlink_metadata(path) {
            metadata.file_type().is_symlink()
        } else {
            false
        }
    }

    // path checker when we need to extend from symlink
    fn check_path(path: &Path) -> PathBuf {
        if Config::is_symlink(path) {
            let parent = path.parent().unwrap_or_else(|| Path::new(""));
            let target = fs::read_link(path).unwrap();
            parent.join(target)
        } else {
            path.to_path_buf()
        }
    }
}
