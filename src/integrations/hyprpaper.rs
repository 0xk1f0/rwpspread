use crate::worker::ResultPaper;
use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

pub fn push(papers: &Vec<ResultPaper>) -> Result<(), String> {
    let instance_id = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| "no HYPRLAND_INSTANCE_SIGNATURE found")?;
    let target_socket = format!("/tmp/hypr/{}/.hyprpaper.sock", instance_id);

    // block till we can connect
    loop {
        match UnixStream::connect(&target_socket) {
            Ok(_) => {
                break;
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(500));
            }
        }
    }

    // try to connect to socket
    let mut socket =
        UnixStream::connect(target_socket).map_err(|_| "hyprpaper: cant connect to socket")?;

    // unload all first
    let mut buffer = [0; 1024];
    socket
        .write_all(b"unload all")
        .map_err(|e| format!("hyprpaper: {}", e))?;
    socket
        .read(&mut buffer)
        .map_err(|e| format!("hyprpaper: {}", e))?;

    if String::from_utf8_lossy(&buffer).to_string().contains("ok") {
        for paper in papers {
            socket
                .write_all(format!("preload {}", paper.full_path).as_bytes())
                .map_err(|e| format!("hyprpaper: {}", e))?;
            socket
                .read(&mut buffer)
                .map_err(|e| format!("hyprpaper: {}", e))?;
            let preload_sum = &buffer[0].saturating_sub(buffer[1]);
            socket
                .write_all(
                    format!("wallpaper {},{}", paper.monitor_name, paper.full_path).as_bytes(),
                )
                .map_err(|e| format!("hyprpaper: {}", e))?;
            socket
                .read(&mut buffer)
                .map_err(|e| format!("hyprpaper: {}", e))?;
            let wallpaper_sum = &buffer[0].saturating_sub(buffer[1]);

            // check for sum, this will ideally be
            // 111 - 107 ("o" + "k") * 2 which is 8
            // since we always expect "ok" on success command
            if wallpaper_sum + preload_sum != 8 {
                return Err("hyprpaper: preload or set failed".to_string());
            }
        }
    } else {
        return Err(String::from_utf8_lossy(&buffer).to_string());
    }

    Ok(())
}
