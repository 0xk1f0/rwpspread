use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::fs::File;
use std::io::Error;

include!("src/parser.rs");

fn completions(outdir: &Path) -> Result<(), Error> {
    let mut app = Args::command();
    generate_to(Shell::Bash, &mut app, "rwpspread", outdir)?;
    generate_to(Shell::Zsh, &mut app, "rwpspread", outdir)?;
    generate_to(Shell::Fish, &mut app, "rwpspread", outdir)?;

    Ok(())
}

fn manpage(outdir: &Path) -> Result<(), Error> {
    let app = Args::command();
    let mut file = File::create(Path::new(&outdir).join("rwpspread.1"))?;
    Man::new(app).render(&mut file)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/parser.rs");

    std::fs::create_dir_all(Path::new("completions"))?;
    completions(Path::new("completions"))?;
    manpage(Path::new("man"))?;

    Ok(())
}
