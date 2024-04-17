use std::{env, process};

// run a one-shot command
pub fn run_oneshot(program: &str, arguments: Vec<&str>) -> Result<(), String> {
    match process::Command::new(program)
        .args(arguments)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                Err(format!("{} returned non-null", program))
            }
        }
        Err(_) => return Err(format!("failed to run {}", program)),
    }
}

// force restart a program
pub fn force_restart(program: &str, arguments: Vec<&str>) -> Result<(), String> {
    match process::Command::new("pidof")
        .arg(program)
        .stdout(process::Stdio::null())
        .status()
    {
        Ok(status) => {
            if status.success() {
                process::Command::new("pkill")
                    .arg(program)
                    .stdout(process::Stdio::null())
                    .output()
                    .map_err(|_| format!("pkill failed: {}", program))?;
            }
        }
        Err(_) => return Err("pidof failed".to_string()),
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
    match process::Command::new("pidof")
        .arg(program)
        .stdout(process::Stdio::null())
        .status()
    {
        Ok(status) => {
            if !status.success() {
                process::Command::new(program)
                    .args(arguments)
                    .stdout(process::Stdio::null())
                    .stderr(process::Stdio::null())
                    .spawn()
                    .map_err(|_| format!("failed to spawn: {}", program))?;
            }
        }
        Err(_) => return Err("pidof failed".to_string()),
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
