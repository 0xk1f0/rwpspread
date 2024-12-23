use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

// alignment enumerator
#[derive(clap::ValueEnum, Clone, Hash)]
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

// locker enumerator
#[derive(clap::ValueEnum, Clone, Hash, PartialEq)]
pub enum Locker {
    Swaylock,
    Hyprlock,
}

impl std::fmt::Display for Locker {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Swaylock => {
                write!(f, "swaylock")
            }
            Self::Hyprlock => {
                write!(f, "hyprlock")
            }
        }
    }
}

// backend enumerator
#[derive(clap::ValueEnum, Clone, Hash, PartialEq)]
pub enum Backend {
    Wpaperd,
    Swaybg,
    Hyprpaper,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Swaybg => {
                write!(f, "swaybg")
            }
            Self::Wpaperd => {
                write!(f, "wpaperd")
            }
            Self::Hyprpaper => {
                write!(f, "hyprpaper")
            }
        }
    }
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
pub struct InitGroup {
    /// Image file or directory path
    #[arg(short, long)]
    image: Option<String>,

    /// Show detectable information
    #[arg(long)]
    info: bool,
}

/// Multi-Monitor Wallpaper Utility
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    init_group: InitGroup,

    /// Output directory path
    #[arg(short, long)]
    output: Option<String>,

    /// Do not downscale the base image, align the layout instead
    #[arg(short, long, value_enum)]
    align: Option<Alignment>,

    /// Wallpaper setter backend
    #[arg(short, long, value_enum)]
    backend: Option<Backend>,

    /// Lockscreen implementation to generate for
    #[arg(short, long, value_enum)]
    locker: Option<Locker>,

    /// Enable daemon mode and resplit on output changes
    #[arg(short, long)]
    daemon: bool,

    /// Generate a color palette from input image
    #[arg(short, long)]
    palette: bool,

    /// Script to execute before splitting
    #[arg(long)]
    pre: Option<String>,

    /// Script to execute after splitting
    #[arg(long)]
    post: Option<String>,

    /// Force resplit, skips all image cache checks
    #[arg(short, long)]
    force_resplit: bool,
}

#[derive(Hash)]
pub struct Config {
    pub input_path: PathBuf,
    pub outdir_path: Option<String>,
    pub backend: Option<Backend>,
    pub locker: Option<Locker>,
    pub daemon: bool,
    pub palette: bool,
    pub force_resplit: bool,
    pub info: bool,
    pub align: Option<Alignment>,
    pub pre_path: Option<String>,
    pub post_path: Option<String>,
    version: String,
}

impl Config {
    pub fn new() -> Result<Option<Self>, String> {
        // handle args
        let args = Args::parse();

        // check for early exit due to info passage
        if args.init_group.image.is_none() && args.init_group.info {
            return Ok(None);
        }

        // get valid input path
        let input_path = Config::to_valid_path(&args.init_group.image.unwrap(), false, false)?;

        // get valid output directory
        let outdir_path: Option<String>;
        if args.output.is_some() {
            // convert to string since we expect one
            let raw_path = Config::to_valid_path(&args.output.unwrap(), false, true)?;
            outdir_path = Some(raw_path.to_string_lossy().trim_end_matches('/').to_string());
        } else {
            // no explicit path specified
            outdir_path = None
        }

        // check for scripts
        let pre_path: Option<String>;
        if args.pre.is_some() {
            pre_path = Some(
                Config::to_valid_path(&args.pre.unwrap(), true, false)?
                    .to_string_lossy()
                    .to_string(),
            );
        } else {
            pre_path = None;
        }
        let post_path: Option<String>;
        if args.post.is_some() {
            post_path = Some(
                Config::to_valid_path(&args.post.unwrap(), true, false)?
                    .to_string_lossy()
                    .to_string(),
            );
        } else {
            post_path = None;
        }

        // get own version
        let version: String = String::from(env!("CARGO_PKG_VERSION"));

        // construct
        Ok(Some(Self {
            input_path,
            outdir_path,
            align: args.align,
            backend: args.backend,
            locker: args.locker,
            daemon: args.daemon,
            palette: args.palette,
            force_resplit: args.force_resplit,
            info: args.init_group.info,
            pre_path,
            post_path,
            version,
        }))
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
    fn extend_path(path: &Path) -> PathBuf {
        if Config::is_symlink(path) {
            let parent = path.parent().unwrap_or_else(|| Path::new(""));
            let target = fs::read_link(path).unwrap();
            parent.join(target)
        } else {
            path.to_path_buf()
        }
    }

    // check if path exists correctly and return if true
    fn to_valid_path(path: &String, file: bool, dir: bool) -> Result<PathBuf, String> {
        let path_buffer = Path::new(path);
        if fs::metadata(path_buffer).is_ok() {
            // evaluate and extend
            // also always canonicalize path so it is absolute
            let corrected_buffer = fs::canonicalize(Config::extend_path(path_buffer))
                .map_err(|_| "could not extend path")?;
            if (file || (!dir && !file)) && fs::metadata(&corrected_buffer).unwrap().is_file() {
                // valid file
                return Ok(corrected_buffer);
            }
            if (dir || (!dir && !file)) && fs::metadata(&corrected_buffer).unwrap().is_dir() {
                // valid directory
                return Ok(corrected_buffer);
            }
        }
        // no metadata, file or dir, consider invalid
        Err(format!(
            "\"{}\": invalid {}",
            path,
            if file {
                "file"
            } else if dir {
                "directory"
            } else {
                "file or directory"
            }
        ))
    }
}
