use crate::worker::ResultPaper;
use std::fs;

pub struct Swaylock;
impl Swaylock {
    // generate new file
    pub fn new(papers: &Vec<ResultPaper>, path: &String) -> Result<(), String> {
        let mut base_string = String::new();
        for paper in papers {
            base_string += &format!("-i {}:{} ", paper.monitor_name, paper.full_path);
        }
        fs::write(format!("{}/rwps_swaylock.conf", path), base_string)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}
