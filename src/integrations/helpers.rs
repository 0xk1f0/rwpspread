use std::process;

// force restart a program
pub fn force_restart(program: &str, arguments: Vec<&str>) -> Result<(), String> {
    match process::Command::new("pidof")
        .args(&[program])
        .stdout(process::Stdio::null())
        .status()
    {
        Ok(status) => {
            if status.success() {
                process::Command::new("killall")
                    .args(&["-9", program])
                    .stdout(process::Stdio::null())
                    .output()
                    .map_err(|err| err.to_string())?;
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    process::Command::new(program)
        .args(arguments)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok(())
}

// soft restart a program
pub fn soft_restart(program: &str, arguments: Vec<&str>) -> Result<(), String> {
    match process::Command::new("pidof")
        .args(&[program])
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
                    .map_err(|err| err.to_string())?;
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    Ok(())
}

// check if a program is available using which
pub fn is_installed(program: &str) -> bool {
    match process::Command::new("which")
        .arg(program)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}
