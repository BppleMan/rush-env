//! rush-say —— 极简、视觉对齐的 shell 注释气泡生成器
//!
//! 适合输出到 shell/zsh 脚本、README、帮助信息。
//!
//! ## 基本用法
//! ```rust
//! use rush_say::say_section;
//! let mut s = String::new();
//! say_section(&mut s, "你好，Rush!\n可自动居中、自动分行。", 48, 2).unwrap();
//! println!("{s}");
//! ```

/// 输出漂亮的注释框气泡（支持自动分行、视觉居中、中文/emoji等宽）
///
/// - `writer`: 输出目标（如 String/stdout）
/// - `content`: 任意多行字符串
/// - `width`/`padding`：可选参数（默认48/2）可自定义
pub fn say_section<W: std::fmt::Write>(writer: &mut W, content: &str, width: usize, padding: usize) -> std::fmt::Result {
    let max_line_width = width - 2 - padding * 2;
    let border = format!("#{}#", "-".repeat(width - 2));
    writeln!(writer, "{border}")?;

    let mut chars = content.chars().peekable();
    loop {
        while chars.peek().is_some() && *chars.peek().unwrap() == '\n' {
            write!(writer, "#")?;
            for _ in 0..(width - 2) {
                write!(writer, " ")?;
            }
            writeln!(writer, "#")?;
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        let mut current = String::new();
        let mut visual = 0;
        while let Some(&ch) = chars.peek() {
            if ch == '\n' {
                break;
            }
            let ch_width = visual_width_char(ch);
            if visual + ch_width > max_line_width {
                break;
            }
            visual += ch_width;
            current.push(ch);
            chars.next();
        }
        if chars.peek() == Some(&'\n') {
            chars.next();
        }
        let spaces = max_line_width - visual;
        let left = padding + spaces / 2;
        let right = padding + (spaces - spaces / 2);
        write!(writer, "#")?;
        for _ in 0..left {
            write!(writer, " ")?;
        }
        write!(writer, "{current}")?;
        for _ in 0..right {
            write!(writer, " ")?;
        }
        writeln!(writer, "#")?;
    }
    writeln!(writer, "{border}")?;
    Ok(())
}

/// 视觉宽度换算（可随时自定义规则）
fn visual_width_char(ch: char) -> usize {
    match ch {
        '\u{4e00}'..='\u{9fff}'   // CJK汉字
        | '\u{3000}'..='\u{303f}' // CJK标点
        | '\u{3040}'..='\u{30ff}' // 日文
        | '\u{ff00}'..='\u{ffef}' // 全角
        => 2,
        '\u{1f300}'..='\u{1f6ff}' | '\u{1f900}'..='\u{1f9ff}' => 2, // emoji
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_single_short_line() {
        let mut buf = String::new();
        say_section(&mut buf, "简单说明", 48, 2).unwrap();
        #[rustfmt::skip]
        let expected =
r#"#----------------------------------------------#
#                   简单说明                   #
#----------------------------------------------#
"#;
        assert_eq!(expected, buf);
    }

    #[test]
    fn test_section_multi_line_and_blank() {
        let mut buf = String::new();
        say_section(&mut buf, "标题\n副标题\n\n多行说明", 48, 2).unwrap();
        println!("{buf}");
        #[rustfmt::skip]
        let expected =
r#"#----------------------------------------------#
#                     标题                     #
#                    副标题                    #
#                                              #
#                   多行说明                   #
#----------------------------------------------#
"#;
        assert_eq!(expected, buf);
    }

    #[test]
    fn test_section_auto_wrap_ascii() {
        let mut buf = String::new();
        say_section(&mut buf, "This is a long, long, long, long sentence that should auto wrap nicely.", 48, 2).unwrap();
        println!("{buf}");
        #[rustfmt::skip]
        let expected =
r#"#----------------------------------------------#
#  This is a long, long, long, long sentence   #
#        that should auto wrap nicely.         #
#----------------------------------------------#
"#;
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_section_utf8_chinese_emoji() {
        let mut buf = String::new();
        say_section(&mut buf, "Rush工具支持emoji🎉，中文分行测试：极其长的一行需要分包到下行", 48, 2).unwrap();
        println!("{buf}");
        #[rustfmt::skip]
        let expected =
r#"#----------------------------------------------#
#  Rush工具支持emoji🎉，中文分行测试：极其长   #
#             的一行需要分包到下行             #
#----------------------------------------------#
"#;
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_section_full_width_chars() {
        let mut buf = String::new();
        say_section(&mut buf, "全角：ＡＢＣＤＥＦ, ABCDEF", 48, 2).unwrap();
        println!("{buf}");
        #[rustfmt::skip]
        let expected =
r#"#----------------------------------------------#
#          全角：ＡＢＣＤＥＦ, ABCDEF          #
#----------------------------------------------#
"#;
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_section_super_long_wrap() {
        let mut buf = String::new();
        say_section(
            &mut buf,
            "本行超长会被自动换行：这是一个很长很长很长很长很长很长很长很长很长很长的句子，用来测试自动包裹",
            48,
            2,
        )
        .unwrap();
        println!("{buf}");
        #[rustfmt::skip]
        let expected =
r#"#----------------------------------------------#
#  本行超长会被自动换行：这是一个很长很长很长  #
#  很长很长很长很长很长很长很长的句子，用来测  #
#                  试自动包裹                  #
#----------------------------------------------#
"#;
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_section_preserve_empty_lines() {
        let mut buf = String::new();
        say_section(&mut buf, "第一行\n\n\n最后一行", 48, 2).unwrap();
        println!("{buf}");
        #[rustfmt::skip]
        let expected =
r#"#----------------------------------------------#
#                    第一行                    #
#                                              #
#                                              #
#                   最后一行                   #
#----------------------------------------------#
"#;
        assert_eq!(buf, expected);
    }
}
