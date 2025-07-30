mod cli;

use crate::cli::{Cli, SubCmd};
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, WrapErr};
use rush_env::core::rush::Rush;
use rush_env::visitor::{Visit, Visitor};
use rush_env::{init_backtrace, init_base_dir};
use rush_say::Section;
use std::io::stdout;
use std::path::{Path, PathBuf};

const TEMPLATE: &str = include_str!("../assets/template/rush.xml");

fn main() -> Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();
    // init_log(&base_dir);

    let executable = Path::new(&std::env::args().next().ok_or_eyre("Executable name not found")?).canonicalize()?;
    println!("# {}", executable.display());

    #[cfg(debug_assertions)]
    let rush_dir = unsafe {
        let rush_dir = std::env::var("DOTDIR").wrap_err("DOTDIR environment variable must be set")?;
        std::env::set_var("RUSH_DIR", &rush_dir);
        PathBuf::from(rush_dir)
    };
    #[cfg(not(debug_assertions))]
    let rush_dir = PathBuf::new(std::env::var("RUSH_DIR").wrap_err("RUSH_DIR environment variable must be set")?);

    let cli = Cli::parse();
    println!("{cli:#?}");

    match cli.sub_cmd {
        None => {
            let rush: Rush = quick_xml::de::from_str(TEMPLATE)?;
            println!("{rush:#?}");
            let mut context = Visitor {
                rush_dir,
                section: Section::new(64, 2),
                ..Default::default()
            };
            rush.visit(&mut context, &mut stdout())?;
        }
        Some(cmd) => cmd.execute(&rush_dir, &executable)?,
    }

    Ok(())
}
