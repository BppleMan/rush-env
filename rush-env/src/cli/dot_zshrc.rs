use color_eyre::eyre::OptionExt;
use std::path::Path;

pub fn generate_dot_zshrc(
    rush_dir: impl AsRef<Path>,
    executable: impl AsRef<Path>,
    writer: &mut impl std::io::Write,
) -> color_eyre::Result<()> {
    writeln!(
        writer,
        "export RUSH_DIR={}",
        rush_dir.as_ref().to_str().ok_or_eyre("RUSH_DIR 不是标准UTF-8可视字符路径")?
    )?;
    writeln!(
        writer,
        "export PATH={}:${{PATH}}",
        executable
            .as_ref()
            .to_str()
            .ok_or_eyre("rush 可执行文件路径不是标准UTF-8可视字符路径")?
    )?;
    writeln!(writer, "source ${{RUSH_DIR}}/rush_zsh.sh")?;
    Ok(())
}
