use crate::worker::ResultPaper;

pub fn new(papers: &Vec<ResultPaper>) -> Result<Vec<&str>, String> {
    let mut arguments: Vec<&str> = Vec::new();
    for paper in papers {
        arguments.push(&"-o");
        arguments.push(&paper.monitor_name);
        arguments.push(&"-i");
        arguments.push(&paper.full_path);
    }

    Ok(arguments)
}
