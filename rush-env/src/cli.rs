use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Cli {
    pub rush_dir: PathBuf,

    #[command(subcommand)]
    pub sub_cmd: SubCmd,
}

#[derive(Debug, Subcommand)]
pub enum SubCmd {
    Init,
    Install,
}
