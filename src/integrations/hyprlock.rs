use std::collections::HashMap;
use std::fs;

pub struct Hyprlock;
impl Hyprlock {
    /// Generate and save new Hyprlock config to disk
    pub fn new(path: &String, wallpapers: &HashMap<String, String>) -> Result<(), String> {
        let mut base_string = String::new();
        for paper in wallpapers {
            // https://wiki.hyprland.org/Hypr-Ecosystem/hyprlock/#background
            base_string += &format!(
                "background {{\n\tmonitor = {}\n\tpath = {}\n}}\n\n",
                paper.0, paper.1
            );
        }
        fs::write(format!("{}/rwps_hyprlock.conf", path), base_string)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}
