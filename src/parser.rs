use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

// alignment enumerator
#[derive(clap::ValueEnum, Debug, Clone, Hash)]
pub enum Alignment {
    Tl, // Top-Left
    Tr, // Top-Right
    Tc, // Top-Centered
    Bl, // Bottom-Left
    Br, // Bottom-Right
    Bc, // Bottom-Centered
    Rc, // Right-Centered
    Lc, // Left-Centered
    C,  // Centered
}

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

    /// Generate swaylock file
    #[arg(short, long)]
    swaylock: bool,

    /// Generate a color palette from Wallpaper
    #[arg(short, long)]
    palette: bool,

    /// Enable Daemon Watchdog mode, will resplit on Output changes
    #[arg(short, long, requires("wpaperd"))]
    daemon: bool,

    /// Force Resplit, skips all Image Cache checks
    #[arg(long)]
    force_resplit: bool,

    /// Do not downscale the Base Image, align the Layout instead
    #[arg(short, long, value_enum)]
    align: Option<Alignment>,
}

#[derive(Hash)]
pub struct Config {
    pub image_path: PathBuf,
    pub with_wpaperd: bool,
    pub with_swaylock: bool,
    pub with_palette: bool,
    pub daemon: bool,
    pub force_resplit: bool,
    pub align: Option<Alignment>,
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
            with_swaylock: args.swaylock,
            with_palette: args.palette,
            daemon: args.daemon,
            force_resplit: args.force_resplit,
            align: args.align,
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
