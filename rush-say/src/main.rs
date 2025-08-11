mod cli;

use crate::cli::border::BorderType;
use crate::cli::comment::CommentType;
use clap::Parser;
use rush_say::Bubble;
use std::io::{self, Read};

/// 终端气泡注释输出工具，支持自动换行、视觉居中、中文emoji宽度处理。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 输出一个示例注释框
    #[arg(long, value_name = "EXAMPLE", default_value_t = false)]
    example: bool,

    /// 直接以参数传入内容（支持多行），不填则自动从标准输入读取
    #[arg(value_name = "TEXT", required = false)]
    text: Option<String>,

    /// 指定注释框宽度
    #[arg(short, long, value_name = "WIDTH", default_value_t = 48)]
    width: usize,

    /// 指定内容与边框的margin
    #[arg(short, long, value_name = "MARGIN", default_value_t = 1)]
    margin: usize,

    /// 指定内容与边框的padding
    #[arg(short, long, value_name = "PADDING", default_value_t = 2)]
    padding: usize,

    /// 指定边框样式
    #[arg(short, long, value_name = "BORDER", value_enum, default_value_t = BorderType::Rounded)]
    border: BorderType,

    /// 指定注释框的注释样式
    #[arg(short, long, value_name = "COMMENT", value_enum)]
    comment: Option<CommentType>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.example {
        let bubble = Bubble::example();
        bubble.say(&mut io::stdout())?;
        return Ok(());
    }

    let bubble = if cli.example {
        Bubble::example()
    }
    // 支持直接参数传入或标准输入（如管道/重定向/多行）
    else if let Some(text) = cli.text {
        Bubble::new(text)
    } else {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        Bubble::new(content)
    };

    let mut bubble = bubble.set_width(cli.width).set_padding(cli.padding).set_border(cli.border.build());
    if let Some(comment_type) = cli.comment {
        bubble = bubble.set_comment(comment_type.build());
    }
    bubble.say(&mut io::stdout())?;
    Ok(())
}
