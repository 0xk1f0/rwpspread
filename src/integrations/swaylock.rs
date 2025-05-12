use std::collections::HashMap;
use std::fs;

pub struct Swaylock;
impl Swaylock {
    /// Build and save a new swaylock config to disk
    pub fn new(path: &String, wallpapers: &HashMap<String, String>) -> Result<(), String> {
        let mut base_string = String::new();
        for paper in wallpapers {
            base_string += &format!("-i {}:{} ", paper.0, paper.1);
        }
        fs::write(format!("{}/rwps_swaylock.conf", path), base_string)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}
