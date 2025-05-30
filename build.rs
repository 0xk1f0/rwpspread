use clap::CommandFactory;
use clap_complete::{Shell, generate_to};
use clap_mangen::Man;
use std::io::Error;
use std::path::Path;

include!("src/cli.rs");

fn completions(outdir: &Path) -> Result<(), Error> {
    let mut app = Args::command();
    generate_to(Shell::Bash, &mut app, "rwpspread", outdir)?;
    generate_to(Shell::Zsh, &mut app, "rwpspread", outdir)?;
    generate_to(Shell::Fish, &mut app, "rwpspread", outdir)?;

    Ok(())
}

fn manpage(outdir: &Path) -> Result<(), Error> {
    let app = Args::command();
    let mut file = fs::File::create(Path::new(&outdir).join("rwpspread.1"))?;
    Man::new(app).render(&mut file)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/cli.rs");

    let outdir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&outdir).ancestors().nth(3).unwrap();
    std::fs::create_dir_all(&dest.join("completions"))?;
    std::fs::create_dir_all(&dest.join("man"))?;
    completions(&dest.join("completions"))?;
    manpage(&dest.join("man"))?;

    Ok(())
}
