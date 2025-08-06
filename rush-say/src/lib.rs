//! rush-say —— 极简、视觉对齐的 shell 注释气泡生成器
//!
//! 适合输出到 shell/zsh 脚本、README、帮助信息。
//!
//! ## 基本用法
//! ```rust
//! use std::io::stdout;
//! use rush_say::say_section;
//! say_section(&mut stdout(), "你好，Rush!\n可自动居中、自动分行。", 48, 2).unwrap();
//! ```

mod border;
pub use border::{BorderStyle, BubbleStyle};
mod bubble;
mod layout;
mod bubble_builder;
mod widget;
mod text;

use crate::border::CommentStyle;
pub use bubble::*;

/// 输出漂亮的注释框气泡（支持自动分行、视觉居中、中文/emoji等宽）
///
/// - `writer`: 输出目标（如 String/stdout）
/// - `content`: 任意多行字符串
/// - `width`/`padding`：可选参数（默认48/2）可自定义
/// - `border_style`: 边框样式（可选，默认注释风格）
pub fn say_section_with_style(writer: &mut impl std::io::Write, content: &str, style: &BubbleStyle) -> std::io::Result<()> {
    // let max_line_width = style.width - 2 - style.padding * 2;
    // let top_border = format!(
    //     "{}{}{}",
    //     style.border_style.top_left,
    //     style.border_style.horizontal.to_string().repeat(style.width - 2),
    //     style.border_style.top_right
    // );
    // writeln!(writer, "{top_border}")?;
    //
    // let mut chars = content.chars().peekable();
    // loop {
    //     while chars.peek().is_some() && *chars.peek().unwrap() == '\n' {
    //         write!(writer, "{}", style.border_style.vertical)?;
    //         for _ in 0..(style.width - 2) {
    //             write!(writer, "{}", style.border_style.vertical)?;
    //         }
    //         writeln!(writer, "{}", style.border_style.vertical)?;
    //         chars.next();
    //     }
    //     if chars.peek().is_none() {
    //         break;
    //     }
    //     let mut current = String::new();
    //     let mut visual = 0;
    //     while let Some(&ch) = chars.peek() {
    //         if ch == '\n' {
    //             break;
    //         }
    //         let ch_width = visual_width_char(ch);
    //         if visual + ch_width > max_line_width {
    //             break;
    //         }
    //         visual += ch_width;
    //         current.push(ch);
    //         chars.next();
    //     }
    //     if chars.peek() == Some(&'\n') {
    //         chars.next();
    //     }
    //     let spaces = max_line_width - visual;
    //     let left = style.padding + spaces / 2;
    //     let right = style.padding + (spaces - spaces / 2);
    //     write!(writer, "{}", style.border_style.vertical)?;
    //     for _ in 0..left {
    //         write!(writer, " ")?;
    //     }
    //     write!(writer, "{current}")?;
    //     for _ in 0..right {
    //         write!(writer, " ")?;
    //     }
    //     writeln!(writer, "{}", style.border_style.vertical)?;
    // }
    // let bottom_border = format!(
    //     "{}{}{}",
    //     style.border_style.bottom_left,
    //     style.border_style.horizontal.to_string().repeat(style.width - 2),
    //     style.border_style.bottom_right
    // );
    // writeln!(writer, "{bottom_border}")?;
    Ok(())
}

pub fn say_section(writer: &mut impl std::io::Write, content: &str, width: usize, padding: usize) -> std::io::Result<()> {
    // let style = BubbleStyle {
    //     width,
    //     padding,
    //     border_style: BorderStyle::default(),
    // };
    let section = Bubble::builder()
        .width(width)
        .padding(padding)
        .border_style(BorderStyle::default())
        .comment_style(CommentStyle::default())
        .build();
    section.say(writer, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_section_single_short_line() {
        let mut buf = Cursor::new(Vec::new());
        say_section(&mut buf, "简单说明", 48, 2).unwrap();
        #[rustfmt::skip]
        let expected = r#"
#----------------------------------------------#
#                   简单说明                   #
#----------------------------------------------#
"#;
        assert_eq!(expected, String::from_utf8(buf.into_inner()).unwrap());
    }

    #[test]
    fn test_section_multi_line_and_blank() {
        let mut buf = Cursor::new(Vec::new());
        say_section(&mut buf, "标题\n副标题\n\n多行说明", 48, 2).unwrap();
        #[rustfmt::skip]
        let expected = r#"
#----------------------------------------------#
#                     标题                     #
#                    副标题                    #
#                                              #
#                   多行说明                   #
#----------------------------------------------#
"#;
        assert_eq!(expected, String::from_utf8(buf.into_inner()).unwrap());
    }

    #[test]
    fn test_section_auto_wrap_ascii() {
        let mut buf = Cursor::new(Vec::new());
        say_section(
            &mut buf,
            "This is a long, long, long, long sentence that should auto wrap nicely.",
            48,
            2,
        )
        .unwrap();
        #[rustfmt::skip]
        let expected = r#"
#----------------------------------------------#
#  This is a long, long, long, long sentence   #
#        that should auto wrap nicely.         #
#----------------------------------------------#
"#;
        assert_eq!(expected, String::from_utf8(buf.into_inner()).unwrap());
    }

    #[test]
    fn test_section_utf8_chinese_emoji() {
        let mut buf = Cursor::new(Vec::new());
        say_section(&mut buf, "Rush工具支持emoji🎉，中文分行测试：极其长的一行需要分包到下行", 48, 2).unwrap();
        #[rustfmt::skip]
        let expected = r#"
#----------------------------------------------#
#  Rush工具支持emoji🎉，中文分行测试：极其长   #
#             的一行需要分包到下行             #
#----------------------------------------------#
"#;
        assert_eq!(expected, String::from_utf8(buf.into_inner()).unwrap());
    }

    #[test]
    fn test_section_full_width_chars() {
        let mut buf = Cursor::new(Vec::new());
        say_section(&mut buf, "全角：ＡＢＣＤＥＦ, ABCDEF", 48, 2).unwrap();
        #[rustfmt::skip]
        let expected = r#"
#----------------------------------------------#
#          全角：ＡＢＣＤＥＦ, ABCDEF          #
#----------------------------------------------#
"#;
        assert_eq!(expected, String::from_utf8(buf.into_inner()).unwrap());
    }

    #[test]
    fn test_section_super_long_wrap() {
        let mut buf = Cursor::new(Vec::new());
        say_section(
            &mut buf,
            "本行超长会被自动换行：这是一个很长很长很长很长很长很长很长很长很长很长的句子，用来测试自动包裹",
            48,
            2,
        )
        .unwrap();
        #[rustfmt::skip]
        let expected =r#"
#----------------------------------------------#
#  本行超长会被自动换行：这是一个很长很长很长  #
#  很长很长很长很长很长很长很长的句子，用来测  #
#                  试自动包裹                  #
#----------------------------------------------#
"#;
        assert_eq!(expected, String::from_utf8(buf.into_inner()).unwrap());
    }

    #[test]
    fn test_section_preserve_empty_lines() {
        let mut buf = Cursor::new(Vec::new());
        say_section(&mut buf, "第一行\n\n\n最后一行", 48, 2).unwrap();
        #[rustfmt::skip]
        let expected = r#"
#----------------------------------------------#
#                    第一行                    #
#                                              #
#                                              #
#                   最后一行                   #
#----------------------------------------------#
"#;
        assert_eq!(expected, String::from_utf8(buf.into_inner()).unwrap());
    }
}
