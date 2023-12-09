use crate::worker::ResultPaper;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process;
use toml;

pub struct Wpaperd {
    pub initial_path: String,
    pub config_hash: String,
    config_path: String,
}

impl Wpaperd {
    pub fn new(initial_path: String, config_hash: String) -> Result<Self, String> {
        // Check if wpaperd is available
        match process::Command::new("/bin/which")
            .arg("wpaperd")
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()
        {
            Ok(status) => {
                if status.success() {
                    Ok(Self {
                        initial_path,
                        config_hash,
                        config_path: format!(
                            "{}/.config/wpaperd/wallpaper.toml",
                            env::var("HOME").unwrap()
                        ),
                    })
                } else {
                    return Err("wpaperd not installed".to_string());
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    // build new wpaperd config to file
    pub fn build(&self, wallpapers: &Vec<ResultPaper>) -> Result<(), String> {
        // Create a new config file
        let mut config_file =
            File::create(&self.config_path).map_err(|_| "unable to open config")?;

        // Open the file
        let read_file =
            std::fs::read_to_string(&self.config_path).map_err(|_| "unable to read config")?;

        // Parse the string into a TOML value
        let mut values = read_file
            .parse::<toml::Value>()
            .map_err(|_| "unable to parse config")?;

        // Add new output sections
        for fragment in wallpapers {
            // insert new section
            values.as_table_mut().unwrap().insert(
                fragment.monitor_name.to_string(),
                toml::Value::Table(Default::default()),
            );
            // add path value
            let path = values.get_mut(fragment.monitor_name.to_string()).unwrap();
            path.as_table_mut().unwrap().insert(
                "path".to_string(),
                toml::Value::String(fragment.full_path.to_string()),
            );
        }

        // write the hash first
        config_file
            .write(
                format!(
                    "# {}\n# DO NOT EDIT! AUTOGENERATED CONFIG!\n\n",
                    self.config_hash
                )
                .as_bytes(),
            )
            .unwrap();

        // input image path for default statement
        config_file
            .write(format!("[default]\npath = \"{}\"\n\n", self.initial_path).as_bytes())
            .unwrap();

        // write actual config
        config_file
            .write_all(toml::to_string_pretty(&values).unwrap().as_bytes())
            .unwrap();

        // return
        Ok(())
    }

    // check for existing config
    pub fn check_existing(&self) -> bool {
        // Open the file
        let read_file = std::fs::read_to_string(&self.config_path).unwrap_or_default();

        // check if we find the correct hash
        if read_file.starts_with(&format!("# {}", self.config_hash)) {
            // hash matches, don't regenerate
            return true;
        }

        false
    }
}

// check if running, if not run
pub fn restart() -> Result<(), String> {
    // Check if there is a running wpaperd process
    match process::Command::new("pidof")
        .args(&["wpaperd"])
        .stdout(process::Stdio::null())
        .status()
    {
        Ok(status) => {
            if status.success() {
                // kill it with fire
                process::Command::new("killall")
                    .args(&["-9", "wpaperd"])
                    .stdout(process::Stdio::null())
                    .output()
                    .map_err(|err| err.to_string())?;
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    // Spawn new wpaperd instance
    process::Command::new("wpaperd")
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok(())
}

// only start if we need to
pub fn soft_restart() -> Result<(), String> {
    // Check if there is a running wpaperd process
    match process::Command::new("pidof")
        .args(&["wpaperd"])
        .stdout(process::Stdio::null())
        .status()
    {
        Ok(status) => {
            if !status.success() {
                // Spawn new wpaperd instance
                process::Command::new("wpaperd")
                    .spawn()
                    .map_err(|err| err.to_string())?;
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    Ok(())
}
