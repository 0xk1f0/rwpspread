use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

pub struct Hyprpaper;
impl Hyprpaper {
    pub fn push(wallpapers: &Vec<(String, String)>) -> Result<(), String> {
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
                    // unload all first and check for success
                    let mut buffer = [0; 1024];
                    socket
                        .write_all(b"unload all")
                        .map_err(|err| format!("hyprpaper: {}", err))?;
                    socket
                        .read(&mut buffer)
                        .map_err(|err| format!("hyprpaper: {}", err))?;
                    if !String::from_utf8_lossy(&buffer)
                        .to_string()
                        .to_lowercase()
                        .contains("ok")
                    {
                        return Err("hyprpaper: unload failed".to_string());
                    }

                    // execute call for every monitor wallpaper
                    for paper in wallpapers {
                        // preload wallpaper and check for success
                        socket
                            .write_all(format!("preload {}", paper.1).as_bytes())
                            .map_err(|err| format!("hyprpaper: {}", err))?;
                        socket
                            .read(&mut buffer)
                            .map_err(|err| format!("hyprpaper: {}", err))?;
                        if !String::from_utf8_lossy(&buffer)
                            .to_string()
                            .to_lowercase()
                            .contains("ok")
                        {
                            return Err("hyprpaper: preload failed".to_string());
                        }

                        // set wallpaper and check for success
                        socket
                            .write_all(format!("wallpaper {},{}", paper.0, paper.1).as_bytes())
                            .map_err(|err| format!("hyprpaper: {}", err))?;
                        socket
                            .read(&mut buffer)
                            .map_err(|err| format!("hyprpaper: {}", err))?;
                        if !String::from_utf8_lossy(&buffer)
                            .to_string()
                            .to_lowercase()
                            .contains("ok")
                        {
                            return Err("hyprpaper: set failed".to_string());
                        }
                    }

                    return Ok(());
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(250));
                }
            }
        }

        Err("hyprpaper: connection timeout reached".to_string())
    }
}
