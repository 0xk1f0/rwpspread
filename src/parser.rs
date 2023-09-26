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

    /// Use wpaperd Integration
    #[arg(short, long)]
    wpaperd: bool,

    /// Generate a color palette from Wallpaper
    #[arg(short, long)]
    palette: bool,

    /// Enable Daemon Watchdog mode, will resplit on Output changes
    #[arg(short, long)]
    daemon: bool,

    /// Force Resplit, skips all Image Cache checks
    #[arg(long)]
    force_resplit: bool,

    /// Don't downscale the Base Image, center the Layout instead
    #[arg(long)]
    center: bool,
}

#[derive(Hash)]
pub struct Config {
    pub image_path: PathBuf,
    pub with_wpaperd: bool,
    pub with_palette: bool,
    pub daemon: bool,
    pub force_resplit: bool,
    pub center: bool,
    version: String,
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

        // get own version
        let version: String = String::from(env!("CARGO_PKG_VERSION"));

        // construct
        Ok(Self {
            image_path: in_path,
            with_wpaperd: args.wpaperd,
            with_palette: args.palette,
            daemon: args.daemon,
            force_resplit: args.force_resplit,
            center: args.center,
            version,
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
