mod cli;

use crate::cli::RushCli;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use rush_env::core::rush::Rush;
use rush_env::core::rush_context::RushContext;
use rush_env::scaffold::Scaffold;
use rush_env::{init_backtrace, init_base_dir, init_log};
use tracing::info;

fn main() -> Result<()> {
    info!("[Rush Env]");
    let cli = RushCli::parse();

    let rush_dir = match &cli.rush_dir {
        Some(dir) => dir.clone(),
        None => init_base_dir(),
    };

    init_backtrace();
    init_log(&rush_dir);
    std::env::set_current_dir(&rush_dir).wrap_err("无法切换到 rush 目录")?;

    let mut context = RushContext::from_env().wrap_err("无法初始化 RushContext")?;
    context.push_var("RUSH_DIR", rush_dir.to_string_lossy());
    let preset: Rush = toml::from_str(Scaffold::rush_toml()?)?;
    println!("{preset:#?}");
    preset.install_plugins(&mut context)?;

    info!("初始化脚手架到目录: {}", rush_dir.display());
    Scaffold::extract_scaffold(&rush_dir)?;

    Ok(())
}
