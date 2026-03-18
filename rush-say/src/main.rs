mod cli;

use crate::cli::border::BorderType;
use crate::cli::comment::CommentType;
use clap::Parser;
use rush_say::bubble::Bubble;
use rush_say::layout::{Align, Axis};
use rush_say::simple_rng::SimpleRng;
use std::io::{self, Read};

/// 终端气泡注释输出工具，支持自动换行、视觉居中、中文emoji宽度处理。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Cli {
    /// 输出一个示例注释框
    #[arg(long, value_name = "EXAMPLE", default_value_t = false)]
    example: bool,

    /// 直接以参数传入内容，支持传多个字符串（多个则渲染为列表气泡）
    #[arg(value_name = "TEXT")]
    text: Vec<String>,

    /// 指定注释框宽度
    #[arg(short, long, value_name = "WIDTH", default_value_t = 46)]
    width: usize,

    /// 指定内容与边框的margin
    #[arg(short, long, value_name = "MARGIN", default_value_t = 1)]
    margin: usize,

    /// 指定内容与边框的padding
    #[arg(short, long, value_name = "PADDING", default_value_t = 1)]
    padding: usize,

    #[arg(short, long, value_enum, default_value_t = Align::Center)]
    align: Align,

    #[arg(short, long, value_enum, default_value_t = Align::Center)]
    text_align: Align,

    /// 如果存在 list 时的主轴
    #[arg(short = 'x', long, value_enum, default_value_t = Axis::Vertical)]
    axis: Axis,

    /// 指定边框样式
    #[arg(short, long, value_name = "BORDER", value_enum, default_value_t = BorderType::rounded)]
    border: BorderType,

    /// 指定注释框的注释样式
    #[arg(short, long, value_name = "COMMENT", value_enum)]
    comment: Option<CommentType>,
}

fn main() -> color_eyre::Result<()> {
    let _ = color_eyre::install();
    let cli = Cli::parse();

    println!("cli: {cli:?}");

    if cli.example {
        say_example()?;
    } else {
        // 1. 准备输出目标
        let mut stdout = io::stdout();

        // 2. 构建气泡注释器
        let mut bubble = Bubble {
            writer: &mut stdout,
            width: cli.width,
            margin: cli.margin,
            padding: cli.padding,
            border: cli.border.build(),
            align: cli.align,
            axis: cli.axis,
            text_align: cli.text_align,
            comment: cli.comment.map(|ct| ct.build()),
        };

        // 3. 准备文本内容
        let texts = if cli.text.is_empty() {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            vec![content.strip_suffix('\n').unwrap_or(&content).to_string()]
        } else {
            cli.text
        };

        // 4. say / say_more
        if texts.len() == 1 {
            bubble.say(&texts[0])?;
        } else {
            bubble.say_more(&texts)?;
        }
    }

    Ok(())
}

pub fn say_example() -> color_eyre::Result<()> {
    let mut stdout = io::stdout();
    let example_text = r#"
- 📄 为复杂配置文件提供清晰的注释气泡框，增强可读性和理解性
- 📝 将注释文本转换为结构化气泡对话框，使大型配置文件更易维护
- 🖌️ 支持多种边框样式自定义（圆角、方形等）， 适应不同配置文件的风格需求
- 💬 提供多种注释风格（如Java文档风格），兼容各类编程语言和配置格式
- 📏 灵活的排版控制系统：
    * 可调整文本宽度，适应不同显示环境
    * 自定义内边距(padding)和外边距(margin)
    * 多种对齐方式（左对齐、居中等）
- 🧰 可输出到任何实现了Write trait的目标，便于集成到各种配置生成工具
- 🔄 支持链式调用API（如.set_width().set_border()），简化使用流程
"#;
    let mut rng = SimpleRng::new();
    loop {
        let x = rng.next_usize(100);
        let mut bubble = Bubble::example(&mut stdout);
        bubble.random_style(x);

        // 写一组输出；失败时判断是否是 BrokenPipe
        match bubble.say(example_text) {
            Ok(()) => {}
            // Err(e) if e.kind() == io::ErrorKind::BrokenPipe => break Ok(()), // 用户在 less 里 q 了
            Err(e) => {
                if let Some(s) = e.source()
                    && s.downcast_ref::<std::io::Error>()
                        .as_ref()
                        .map(|e| e.kind() == io::ErrorKind::BrokenPipe)
                        .unwrap_or(false)
                {
                    break Ok(());
                } else {
                    return Err(e);
                }
            }
        }
    }
}
