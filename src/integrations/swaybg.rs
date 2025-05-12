use std::collections::HashMap;

pub struct Swaybg;
impl Swaybg {
    pub fn new(wallpapers: &HashMap<String, String>) -> Result<Vec<&str>, String> {
        let mut arguments: Vec<&str> = Vec::new();
        for paper in wallpapers {
            arguments.push(&"-o");
            arguments.push(&paper.0);
            arguments.push(&"-i");
            arguments.push(&paper.1);
        }

        Ok(arguments)
    }
}
