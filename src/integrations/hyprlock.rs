use crate::worker::ResultPaper;
use std::fs;

pub struct Hyprlock;
impl Hyprlock {
    // generate new hyprlock config
    pub fn new(papers: &Vec<ResultPaper>, path: &String) -> Result<(), String> {
        let mut base_string = String::new();
        for paper in papers {
            // push according to hyprlang
            // https://wiki.hyprland.org/Hypr-Ecosystem/hyprlock/#background
            base_string += &format!(
                "background {{\n\tmonitor = {}\n\tpath = {}\n}}\n\n",
                paper.monitor_name, paper.full_path
            );
        }
        fs::write(format!("{}/rwps_hyprlock.conf", path), base_string)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}
