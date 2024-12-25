use crate::worker::ResultPaper;
use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

pub struct Hyprpaper;
impl Hyprpaper {
    pub fn push(papers: &Vec<ResultPaper>) -> Result<(), String> {
        // find socket base with fallback
        let socket_base: String;
        if let Ok(xdg_dir) = env::var("XDG_RUNTIME_DIR") {
            socket_base = xdg_dir;
        } else if let Ok(uid) = env::var("UID") {
            socket_base = format!("/run/user/{}", uid);
        } else {
            return Err("hyprpaper: no valid socket path found".to_string());
        }

        // set target socket with fallback
        let target_socket: String;
        if let Ok(instance_id) = env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            target_socket = format!("{}/hypr/{}/.hyprpaper.sock", socket_base, instance_id)
        } else {
            target_socket = format!("{}/hypr/.hyprpaper.sock", socket_base)
        }

        // block till we can connect or met retry limit
        for _ in 0..40 {
            match UnixStream::connect(&target_socket) {
                Ok(mut socket) => {
                    // unload all first
                    let mut buffer = [0; 1024];
                    socket
                        .write_all(b"unload all")
                        .map_err(|err| format!("hyprpaper: {}", err))?;
                    socket
                        .read(&mut buffer)
                        .map_err(|err| format!("hyprpaper: {}", err))?;

                    if String::from_utf8_lossy(&buffer).to_string().contains("ok") {
                        for paper in papers {
                            socket
                                .write_all(format!("preload {}", paper.full_path).as_bytes())
                                .map_err(|err| format!("hyprpaper: {}", err))?;
                            socket
                                .read(&mut buffer)
                                .map_err(|err| format!("hyprpaper: {}", err))?;
                            let preload_sum = &buffer[0].saturating_sub(buffer[1]);
                            socket
                                .write_all(
                                    format!("wallpaper {},{}", paper.monitor_name, paper.full_path)
                                        .as_bytes(),
                                )
                                .map_err(|err| format!("hyprpaper: {}", err))?;
                            socket
                                .read(&mut buffer)
                                .map_err(|err| format!("hyprpaper: {}", err))?;
                            let wallpaper_sum = &buffer[0].saturating_sub(buffer[1]);

                            // check for sum, this will ideally be
                            // 111 - 107 ("o" + "k") * 2 which is 8
                            // since we always expect "ok" on success command
                            if wallpaper_sum + preload_sum != 8 {
                                return Err("hyprpaper: preload or set failed".to_string());
                            }
                        }
                    } else {
                        return Err("hyprpaper: unexpected socket response".to_string());
                    }

                    return Ok(());
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(250));
                }
            }
        }

        Err("hyprpaper: no connection after 40 tries".to_string())
    }
}
