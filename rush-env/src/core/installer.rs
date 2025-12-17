use crate::core::rush_context::RushContext;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use rush_var::expand_var;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::process::{Child, Command, Stdio};
use std::thread;
use tracing::{error, info};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
#[serde(rename_all = "snake_case")]
pub struct Install {
    pub installer: Installer,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Installer {
    #[default]
    Builtin,
    Git(GitInstaller),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GitInstaller {
    pub url: String,
    pub dest: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
}

impl Install {
    pub fn install(&self, context: &mut RushContext) -> Result<()> {
        match &self.installer {
            Installer::Builtin => info!("使用内置安装器, 无需操作"),
            Installer::Git(git) => git.install(context)?,
        };
        Ok(())
    }
}

impl GitInstaller {
    pub fn install(&self, context: &mut RushContext) -> Result<()> {
        let git = context.which("git").ok_or(eyre!("Git 安装器依赖 Git 可执行文件"))?;
        let dest = expand_var(&self.dest, context);
        info!("开始使用 Git 克隆仓库:");
        info!("{} clone --depth 1 {} {}", git.display(), self.url, dest);
        let mut child = Command::new(&git)
            .args(["clone", "--depth", "1", &self.url, &dest])
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let lines = Self::wait(&mut child)?;
        let status = child.wait()?;
        if !status.success() {
            error!("Git 克隆失败，状态码: {}", status);
            for line in lines {
                match line {
                    Line::Stdout(line) => info!("[stdout] {}", line),
                    Line::Stderr(line) => error!("[stderr] {}", line),
                }
            }
            return Err(eyre!("Git 克隆失败，状态码: {}", status));
        } else {
            info!("Git 克隆成功: {}", self.url);
        }

        if let Some(branch) = self.branch.as_ref().or(self.tag.as_ref().or(self.rev.as_ref())) {
            let mut child = Command::new(git)
                .args(["-C", &dest, "checkout", branch])
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let lines = Self::wait(&mut child)?;
            let status = child.wait()?;
            if !status.success() {
                for line in lines {
                    match line {
                        Line::Stdout(line) => info!("[stdout] {}", line),
                        Line::Stderr(line) => error!("[stderr] {}", line),
                    }
                }
                return Err(eyre!("Git 检出失败，状态码: {}", status));
            } else {
                info!("Git 检出成功: {}", branch);
            }
        }

        Ok(())
    }

    pub fn wait(child: &mut Child) -> Result<Vec<Line>> {
        let (tx, rx) = std::sync::mpsc::channel::<Line>();
        if let Some(stdout) = child.stdout.take() {
            let tx = tx.clone();
            thread::spawn(move || {
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    tx.send(Line::Stdout(line)).unwrap();
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            let tx = tx.clone();
            thread::spawn(move || {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    tx.send(Line::Stderr(line)).unwrap();
                }
            });
        }
        drop(tx);

        Ok(rx.iter().collect::<Vec<_>>())
    }
}

enum Line {
    Stdout(String),
    Stderr(String),
}
