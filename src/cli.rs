use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// alignment enumerator
#[derive(clap::ValueEnum, Clone, Serialize)]
pub enum Alignment {
    Tl, // Top-Left
    Tr, // Top-Right
    Tc, // Top-Centered
    Bl, // Bottom-Left
    Br, // Bottom-Right
    Bc, // Bottom-Centered
    Rc, // Right-Centered
    Lc, // Left-Centered
    Ct, // Centered
}

// locker enumerator
#[derive(clap::ValueEnum, Clone, Serialize, PartialEq)]
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
#[derive(clap::ValueEnum, Clone, Serialize, PartialEq)]
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
#[command(version, about, long_about = None, help_template = "\
{name} {version} - {about}

{usage-heading}
  {usage}

{all-args}
")]
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

    /// Bezel amount in pixels to compensate for
    #[arg(long)]
    bezel: Option<u32>,

    /// List of monitor containing their diagonal in inches [format: "<NAME>:<INCHES>"]
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    monitors: Option<Vec<String>>,

    /// Compensate for different monitor ppi values
    #[arg(long, requires = "monitors")]
    ppi: bool,

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

    /// Watch for wallpaper source changes and resplit on changes
    #[arg(short, long, requires = "daemon")]
    watch: bool,

    /// Force resplit, skips all image cache checks
    #[arg(short, long)]
    force_resplit: bool,
}

#[derive(Serialize)]
pub struct Config {
    pub input_path: PathBuf,
    pub raw_input_path: PathBuf,
    pub output_path: Option<String>,
    pub backend: Option<Backend>,
    pub locker: Option<Locker>,
    pub bezel: Option<u32>,
    pub diagonals: HashMap<String, u32>,
    pub ppi: bool,
    pub daemon: bool,
    pub palette: bool,
    pub force_resplit: bool,
    pub info: bool,
    pub align: Option<Alignment>,
    pub pre_path: Option<String>,
    pub post_path: Option<String>,
    pub watch: bool,
    version: String,
}

impl Config {
    /// Generate and return a new Config based on cli input
    pub fn new() -> Result<Option<Self>, String> {
        // handle args
        let mut args = Args::parse();

        // get valid input path
        if let Some(image_path) = args.init_group.image {
            let input_paths = Config::to_valid_paths(&image_path, false, false)?;

            // get valid output directory
            if let Some(output_path) = args.output {
                // convert to string since we expect one
                args.output = Some(
                    Config::to_valid_paths(&output_path, false, true)?
                        .1
                        .to_string_lossy()
                        .trim_end_matches('/')
                        .to_string(),
                );
            } else {
                // no explicit path specified
                args.output = None
            }

            // check for monitor definitions
            let mut diagonals: HashMap<String, u32> = HashMap::new();
            if let Some(monitors) = args.monitors {
                for entity in monitors {
                    let parts: Vec<&str> = entity.split(":").collect();
                    if parts.len() == 2 {
                        let name = parts[0].trim();
                        let inches_str = parts[1].trim();
                        if let Ok(inches) = inches_str.parse::<u32>() {
                            diagonals.insert(name.to_owned(), inches);
                        }
                    }
                }
            }

            // check for scripts
            if let Some(pre_script_path) = args.pre {
                args.pre = Some(
                    Config::to_valid_paths(&pre_script_path, true, false)?
                        .1
                        .to_string_lossy()
                        .to_string(),
                );
            } else {
                args.pre = None;
            }

            if let Some(post_script_path) = args.post {
                args.post = Some(
                    Config::to_valid_paths(&post_script_path, true, false)?
                        .1
                        .to_string_lossy()
                        .to_string(),
                );
            } else {
                args.post = None;
            }

            Ok(Some(Self {
                input_path: input_paths.1,
                raw_input_path: input_paths.0,
                diagonals: diagonals,
                output_path: args.output,
                align: args.align,
                backend: args.backend,
                locker: args.locker,
                bezel: args.bezel,
                ppi: args.ppi,
                daemon: args.daemon,
                palette: args.palette,
                force_resplit: args.force_resplit,
                info: args.init_group.info,
                pre_path: args.pre,
                post_path: args.post,
                watch: args.watch,
                version: String::from(env!("CARGO_PKG_VERSION")),
            }))
        } else {
            return Ok(None);
        }
    }
    // check if path exists correctly and return if true
    fn to_valid_paths(path: &String, file: bool, dir: bool) -> Result<(PathBuf, PathBuf), String> {
        let raw_path = PathBuf::from(path);
        if fs::metadata(&raw_path).is_ok() {
            // canonicalize path so it is absolute
            let abs_path =
                fs::canonicalize(&raw_path).map_err(|_| "could not canonicalize path")?;
            if (file || (!dir && !file))
                && fs::metadata(&abs_path)
                    .map_err(|_| "could not get metadata")?
                    .is_file()
            {
                // valid file
                return Ok((raw_path, abs_path));
            }
            if (dir || (!dir && !file))
                && fs::metadata(&abs_path)
                    .map_err(|_| "could not get metadata")?
                    .is_dir()
            {
                // valid directory
                return Ok((raw_path, abs_path));
            }
        }
        // no metadata, file or dir, consider invalid
        Err(format!(
            "\"{}\": invalid {}",
            raw_path.display(),
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
