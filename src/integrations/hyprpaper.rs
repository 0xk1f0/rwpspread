use hyprwire_rs::client::HyprWireClient;
use hyprwire_rs::wire;
use std::collections::HashMap;
use std::env;
use std::thread;
use std::time::Duration;

const SUPPORTED_VERSION: u32 = 1;

pub struct Hyprpaper;
impl Hyprpaper {
    /// Generate and push appropriate socket commands to hyprpaper given new input wallpapers
    pub fn push(wallpapers: &HashMap<String, String>) -> Result<(), String> {
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
            match HyprWireClient::connect(&target_socket) {
                Ok(mut client) => {
                    // perform the handshake
                    let protocols = client.perform_handshake(SUPPORTED_VERSION)?;

                    // bind to first available protocol
                    let object_id: u32;
                    if let Some(protocol) = protocols.first() {
                        object_id = client.bind_protocol(&protocol.spec)?;
                    } else {
                        return Err("no protocol to bind to".to_string());
                    }

                    // apply for every wallpaper
                    for paper in wallpapers {
                        // request the new object
                        client.send_message(
                            wire::Code::HW_GENERIC_PROTOCOL_MESSAGE,
                            &[
                                wire::Value::Object(object_id),
                                wire::Value::Uint(0),
                                wire::Value::Seq(client.get_sequence()),
                            ],
                        )?;

                        // process new object handle
                        let response = client.read_message()?;
                        if response.code != wire::Code::HW_NEW_OBJECT {
                            return Err(
                                "expected new object response after object request".to_string()
                            );
                        }
                        let Some(wire::Value::Uint(hp_object_id)) = response.args.get(0) else {
                            return Err(
                                "expected new object for hyprpaper wallpaper object".to_string()
                            );
                        };

                        // set wallpaper path
                        client.send_message(
                            wire::Code::HW_GENERIC_PROTOCOL_MESSAGE,
                            &[
                                wire::Value::Object(*hp_object_id),
                                wire::Value::Uint(0),
                                wire::Value::Varchar(paper.1.to_string()),
                                wire::Value::Seq(client.get_sequence()),
                            ],
                        )?;
                        // set monitor
                        client.send_message(
                            wire::Code::HW_GENERIC_PROTOCOL_MESSAGE,
                            &[
                                wire::Value::Object(*hp_object_id),
                                wire::Value::Uint(2),
                                wire::Value::Varchar(paper.0.to_string()),
                                wire::Value::Seq(client.get_sequence()),
                            ],
                        )?;
                        // set fit mode
                        client.send_message(
                            wire::Code::HW_GENERIC_PROTOCOL_MESSAGE,
                            &[
                                wire::Value::Object(*hp_object_id),
                                wire::Value::Uint(1),
                                wire::Value::Uint(1),
                                wire::Value::Seq(client.get_sequence()),
                            ],
                        )?;
                        // apply
                        client.send_message(
                            wire::Code::HW_GENERIC_PROTOCOL_MESSAGE,
                            &[
                                wire::Value::Object(*hp_object_id),
                                wire::Value::Uint(3),
                                wire::Value::Seq(client.get_sequence()),
                            ],
                        )?;
                        _ = client.read_message()?;
                    }

                    // try to disconnect properly, but dont panic if we cant
                    client.disconnect().unwrap_or(());

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
