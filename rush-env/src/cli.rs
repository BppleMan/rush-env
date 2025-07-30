mod dot_zshrc;

use crate::cli::dot_zshrc::generate_dot_zshrc;
use clap::{Parser, Subcommand};
use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
pub struct Cli {
    pub rush_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub sub_cmd: Option<SubCmd>,
}

#[derive(Debug, Subcommand)]
pub enum SubCmd {
    Init,
    Install,
}

impl SubCmd {
    pub fn execute(self, rush_dir: impl AsRef<Path>, executable: impl AsRef<Path>) -> Result<()> {
        match self {
            SubCmd::Init => {
                // 这里可以添加初始化逻辑
            }
            SubCmd::Install => {
                // 这里可以添加安装逻辑
            }
        }
        Ok(())
    }

    pub fn init(rush_dir: impl AsRef<Path>, executable: impl AsRef<Path>) -> Result<()> {
        let home = std::env::var("HOME").wrap_err("HOME environment variable must be set")?;
        let home_path = Path::new(&home);
        match OpenOptions::new().create_new(true).append(false).open(home_path.join(".zshrc")) {
            Ok(mut dot_zshrc) => {
                generate_dot_zshrc(rush_dir, executable, &mut dot_zshrc)?;
                Ok(())
            }
            Err(e) => {
                if matches!(e.kind(), ErrorKind::AlreadyExists) {
                    eprintln!("Warning: .zshrc already exists.");
                }
                Err(e)?
            }
        }
    }
}
