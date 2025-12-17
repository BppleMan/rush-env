use color_eyre::Result;
use color_eyre::eyre::eyre;
use include_dir::{Dir, DirEntry, include_dir};
use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace};

static SCAFFOLD: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/scaffold");

pub struct Scaffold {}

impl Scaffold {
    pub fn zdotdir(base_dir: impl AsRef<Path>) -> PathBuf {
        base_dir.as_ref().join("zdotdir")
    }

    pub fn zshenv(base_dir: impl AsRef<Path>) -> PathBuf {
        Self::zdotdir(base_dir).join(".zshenv")
    }

    pub fn zshrc(base_dir: impl AsRef<Path>) -> PathBuf {
        Self::zdotdir(base_dir).join(".zshrc")
    }

    pub fn rush_toml() -> Result<&'static str> {
        SCAFFOLD
            .get_file("rush.toml")
            .ok_or(eyre!("脚手架模板中未找到 rush.toml"))?
            .contents_utf8()
            .ok_or(eyre!("脚手架模板中的 rush.toml 不是有效的 UTF-8 文本"))
    }

    pub fn extract_scaffold(rush_dir: impl AsRef<Path>) -> Result<()> {
        let base_path = rush_dir.as_ref();

        let mut dir_builder = DirBuilder::new();
        dir_builder.recursive(true);
        let mut open_options = OpenOptions::new();
        open_options.create(true).write(true).truncate(true);

        Self::walk_scaffold(|entry, depth| {
            if entry.path().ends_with(".DS_Store") {
                trace!("跳过文件: {}[F] {}", "  ".repeat(depth), entry.path().display());
                return Ok(());
            }
            let path = base_path.join(entry.path());
            match entry {
                DirEntry::Dir(_) => {
                    debug!("创建目录: {}[D] {}", "  ".repeat(depth), path.display());
                    dir_builder.create(path)?;
                }
                DirEntry::File(f) => {
                    debug!("创建文件: {}[F] {}", "  ".repeat(depth), path.display());
                    open_options.open(path)?.write_all(f.contents())?;
                }
            }
            Ok(())
        })?;

        Self::gen_dot_zshrc(&rush_dir)?;

        Self::gen_dot_zshenv(&rush_dir)?;

        Ok(())
    }

    fn walk_scaffold<F>(visit: F) -> Result<()>
    where
        F: Fn(&DirEntry, usize) -> Result<()>,
    {
        let mut stack: Vec<(&DirEntry, usize)> = SCAFFOLD.entries().iter().map(|e| (e, 0)).collect();
        // 栈模拟的深度优先遍历
        while let Some((entry, depth)) = stack.pop() {
            match entry {
                DirEntry::Dir(sub_dir) => {
                    visit(entry, depth)?;
                    sub_dir.entries().iter().rev().map(|e| (e, depth + 1)).for_each(|e| stack.push(e));
                }
                DirEntry::File(_file) => {
                    visit(entry, depth)?;
                }
            }
        }

        Ok(())
    }

    pub fn gen_dot_zshrc(rush_dir: impl AsRef<Path>) -> Result<()> {
        let path = Self::zshrc(&rush_dir);
        let mut buffer = String::new();
        use std::fmt::Write;

        writeln!(&mut buffer, "export RUSH_DIR={}", rush_dir.as_ref().display())?;
        writeln!(&mut buffer, "source ${{RUSH_DIR}}/rush.sh")?;

        info!("写入文件: {}", path.display());
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?
            .write_all(buffer.as_bytes())?;
        Ok(())
    }

    pub fn gen_dot_zshenv(rush_dir: impl AsRef<Path>) -> Result<()> {
        let mut buffer = String::new();
        use std::fmt::Write;

        writeln!(&mut buffer, "export ZDOTDIR={}", Self::zdotdir(&rush_dir).display())?;

        let path = std::env::home_dir().ok_or(eyre!("无法获取用户主目录"))?.join(".zshenv");
        info!("写入文件: {}", path.display());
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?
            .write_all(buffer.as_bytes())?;

        let path = Self::zshenv(&rush_dir);
        info!("写入文件: {}", path.display());
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?
            .write_all(buffer.as_bytes())?;
        Ok(())
    }
}
