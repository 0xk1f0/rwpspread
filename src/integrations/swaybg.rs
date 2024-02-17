use crate::worker::ResultPaper;
use std::process;

fn restart(arguments: Vec<&str>) -> Result<(), String> {
    // Check if swaybg is running
    match process::Command::new("pidof")
        .args(&["swaybg"])
        .stdout(process::Stdio::null())
        .status()
    {
        Ok(status) => {
            if status.success() {
                // kill it with fire
                process::Command::new("killall")
                    .args(&["-9", "swaybg"])
                    .stdout(process::Stdio::null())
                    .output()
                    .map_err(|err| err.to_string())?;
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    // Spawn new wpaperd instance
    process::Command::new("swaybg")
        .args(arguments)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok(())
}

pub fn run(papers: &Vec<ResultPaper>) -> Result<(), String> {
    // Check if swaybg is available
    match process::Command::new("which")
        .arg("swaybg")
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
    {
        Ok(status) => {
            if status.success() {
                let mut process_arguments: Vec<&str> = Vec::new();
                for paper in papers {
                    process_arguments.push(&"-o");
                    process_arguments.push(&paper.monitor_name);
                    process_arguments.push(&"-i");
                    process_arguments.push(&paper.full_path);
                }
                restart(process_arguments)
            } else {
                Err("swaybg not installed".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
