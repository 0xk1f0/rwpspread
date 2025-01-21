use std::{env, process};

pub struct Helpers;
impl Helpers {
    // run a one-shot command
    pub fn run_oneshot(program: &str) -> Result<(), String> {
        match process::Command::new(program)
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .output()
        {
            Ok(_) => Ok(()),
            Err(_) => return Err(format!("failed to run {}", program)),
        }
    }

    // check if a process pid exists
    pub fn pid_exists(program: &str) -> Result<bool, String> {
        match process::Command::new("pidof")
            .arg(program)
            .stdout(process::Stdio::null())
            .status()
        {
            Ok(status) => {
                return Ok(status.success());
            }
            Err(_) => return Err("pidof failed".to_string()),
        }
    }

    // force restart a program
    pub fn force_restart(program: &str, arguments: Vec<&str>) -> Result<(), String> {
        if Self::pid_exists(program)? {
            process::Command::new("pkill")
                .arg(program)
                .stdout(process::Stdio::null())
                .output()
                .map_err(|_| format!("pkill failed: {}", program))?;
        }

        process::Command::new(program)
            .args(arguments)
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn()
            .map_err(|_| format!("failed to spawn: {}", program))?;

        Ok(())
    }

    // soft restart a program
    pub fn soft_restart(program: &str, arguments: Vec<&str>) -> Result<(), String> {
        if !Self::pid_exists(program)? {
            process::Command::new(program)
                .args(arguments)
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()
                .map_err(|_| format!("failed to spawn: {}", program))?;
        }

        Ok(())
    }

    // check if a program is available
    pub fn is_installed(program: &str) -> bool {
        if let Some(path) = env::var_os("PATH") {
            for path in env::split_paths(&path) {
                let full_path = path.join(program);
                if full_path.exists() {
                    return true;
                }
            }
        }

        false
    }
}
