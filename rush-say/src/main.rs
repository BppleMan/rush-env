use clap::Parser;
use std::io::{self, Read};

/// 终端气泡注释输出工具，支持自动换行、视觉居中、中文emoji宽度处理。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 直接以参数传入内容（支持多行），不填则自动从标准输入读取
    #[arg(value_name = "TEXT", required = false)]
    text: Option<String>,

    /// 指定注释框宽度
    #[arg(short, long, value_name = "WIDTH", default_value_t = 48)]
    width: usize,

    /// 指定内容与边框的padding
    #[arg(short, long, value_name = "PADDING", default_value_t = 2)]
    padding: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    let mut content = String::new();
    // 支持直接参数传入或标准输入（如管道/重定向/多行）
    if let Some(text) = cli.text {
        content = text;
    } else {
        io::stdin().read_to_string(&mut content)?;
        content = content.trim_end().to_string(); // 去掉末尾多余空行
    }

    let mut buf = String::new();
    rush_say::say_section(&mut buf, &content, cli.width, cli.padding)?;
    println!("{buf}");
    Ok(())
}
