use std::io::Write;
use std::str::FromStr;

use crate::widget::{Align, Constraints, Size, Widget};
use crate::{Grapheme, Graphemes};

#[derive(Default, Debug, Clone)]
pub struct Text {
    pub content: Graphemes,
    pub align: Align,
    layout: TextLayout,
}

#[derive(Default, Debug, Clone)]
struct TextLayout {
    lines: Vec<Grapheme>,
    size: Size,
}

impl Text {
    pub fn new(content: impl AsRef<str>) -> Text {
        Self::with_align(content, Align::Center)
    }

    pub fn with_align(content: impl AsRef<str>, align: Align) -> Text {
        let content = Graphemes::new(content.as_ref().to_string());
        Text {
            content,
            align,
            ..Default::default()
        }
    }

    pub fn set_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    fn wrap_text(&self, max_width: usize) -> Vec<Grapheme> {
        let mut wrapped_lines: Vec<Grapheme> = vec![];
        let mut line_buffer: Grapheme = Grapheme::default();
        for grapheme in &self.content {
            match grapheme.as_str() {
                "\n" | "\r" | "\r\n" => {
                    wrapped_lines.push(std::mem::take(&mut line_buffer));
                }
                _ => {
                    if line_buffer.ascii_width() + grapheme.ascii_width() > max_width {
                        // 如果当前行宽度超过限制，则换行
                        wrapped_lines.push(std::mem::take(&mut line_buffer));
                    }
                    line_buffer += grapheme.grapheme();
                }
            }
        }
        wrapped_lines.push(line_buffer);
        wrapped_lines
    }
}

impl Widget for Text {
    fn size(&self) -> Size {
        self.layout.size
    }

    fn layout(&mut self, constraints: Constraints) {
        self.layout.lines = self.wrap_text(constraints.max_width);
        let width = self.layout.lines.iter().map(Grapheme::ascii_width).max().unwrap_or(0);
        let height = self.layout.lines.len();
        self.layout.size = Size { width, height };
    }

    fn render(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        let line = &self.layout.lines[row];
        let rendered_line = line.render(&self.content);
        let remaining_width = self.layout.size.width.saturating_sub(line.ascii_width());
        let (left, right) = match self.align {
            Align::Center => (remaining_width / 2, remaining_width - (remaining_width / 2)),
            Align::Left => (0, remaining_width),
            Align::Right => (remaining_width, 0),
        };
        write!(writer, "{}{}{}", " ".repeat(left), rendered_line, " ".repeat(right))
    }
}

impl FromStr for Text {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Text::new(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::widget::text::Text;

    #[test]
    fn test_text_widget_wraps_crlf_and_emoji() {
        let text = Text::new("Hel🧑‍🚀lo,\r\nworld!");
        let lines = text.wrap_text(4);
        assert_eq!(5, lines.len());

        assert_eq!(3, lines[0].ascii_width());
        assert_eq!("Hel", lines[0].render(&text.content));

        assert_eq!(4, lines[1].ascii_width());
        assert_eq!("🧑‍🚀lo", lines[1].render(&text.content));

        assert_eq!(1, lines[2].ascii_width());
        assert_eq!(",", lines[2].render(&text.content));

        assert_eq!(4, lines[3].ascii_width());
        assert_eq!("worl", lines[3].render(&text.content));

        assert_eq!(2, lines[4].ascii_width());
        assert_eq!("d!", lines[4].render(&text.content));
    }

    #[test]
    fn test_text_widget_simple_ascii() {
        let text = Text::new("hello world");
        let lines = text.wrap_text(5);
        assert_eq!(3, lines.len());

        assert_eq!(5, lines[0].ascii_width());
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(5, lines[1].ascii_width());
        assert_eq!(" worl", lines[1].render(&text.content));

        assert_eq!(1, lines[2].ascii_width());
        assert_eq!("d", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_exact_fit() {
        let text = Text::new("hello");
        let lines = text.wrap_text(5);
        assert_eq!(1, lines.len());

        assert_eq!(5, lines[0].ascii_width());
        assert_eq!("hello", lines[0].render(&text.content));
    }

    #[test]
    fn test_text_widget_single_char_per_line() {
        let text = Text::new("abc");
        let lines = text.wrap_text(1);
        assert_eq!(3, lines.len());

        assert_eq!(1, lines[0].ascii_width());
        assert_eq!("a", lines[0].render(&text.content));

        assert_eq!(1, lines[1].ascii_width());
        assert_eq!("b", lines[1].render(&text.content));

        assert_eq!(1, lines[2].ascii_width());
        assert_eq!("c", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_empty_string() {
        let text = Text::new("");
        let lines = text.wrap_text(10);
        assert_eq!(1, lines.len());

        assert_eq!(0, lines[0].ascii_width());
        assert_eq!("", lines[0].render(&text.content));
    }

    #[test]
    fn test_text_widget_single_newline() {
        let text = Text::new("hello\nworld");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].ascii_width());
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(5, lines[1].ascii_width());
        assert_eq!("world", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_multiple_newlines() {
        let text = Text::new("a\n\nb");
        let lines = text.wrap_text(10);
        assert_eq!(3, lines.len());

        assert_eq!(1, lines[0].ascii_width());
        assert_eq!("a", lines[0].render(&text.content));

        assert_eq!(0, lines[1].ascii_width());
        assert_eq!("", lines[1].render(&text.content));

        assert_eq!(1, lines[2].ascii_width());
        assert_eq!("b", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_unicode_wide_chars() {
        let text = Text::new("你好世界");
        let lines = text.wrap_text(4);
        assert_eq!(2, lines.len());

        assert_eq!(4, lines[0].ascii_width());
        assert_eq!("你好", lines[0].render(&text.content));

        assert_eq!(4, lines[1].ascii_width());
        assert_eq!("世界", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_mixed_ascii_unicode() {
        let text = Text::new("hello你好");
        let lines = text.wrap_text(6);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].ascii_width());
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(4, lines[1].ascii_width());
        assert_eq!("你好", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_cr_line_ending() {
        let text = Text::new("line1\rline2");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].ascii_width());
        assert_eq!("line1", lines[0].render(&text.content));

        assert_eq!(5, lines[1].ascii_width());
        assert_eq!("line2", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_trailing_newline() {
        let text = Text::new("hello\n");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].ascii_width());
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(0, lines[1].ascii_width());
        assert_eq!("", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_very_long_word() {
        let text = Text::new("supercalifragilisticexpialidocious");
        let lines = text.wrap_text(10);
        assert_eq!(4, lines.len());

        assert_eq!(10, lines[0].ascii_width());
        assert_eq!("supercalif", lines[0].render(&text.content));

        assert_eq!(10, lines[1].ascii_width());
        assert_eq!("ragilistic", lines[1].render(&text.content));

        assert_eq!(10, lines[2].ascii_width());
        assert_eq!("expialidoc", lines[2].render(&text.content));

        assert_eq!(4, lines[3].ascii_width());
        assert_eq!("ious", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_spaces_and_wrapping() {
        let text = Text::new("a  b  c");
        let lines = text.wrap_text(3);
        assert_eq!(3, lines.len());

        assert_eq!(3, lines[0].ascii_width());
        assert_eq!("a  ", lines[0].render(&text.content));

        assert_eq!(3, lines[1].ascii_width());
        assert_eq!("b  ", lines[1].render(&text.content));

        assert_eq!(1, lines[2].ascii_width());
        assert_eq!("c", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_narrow_width() {
        let text = Text::new("Hel🧑‍🚀lo,wo🧑‍🚀rld!");
        let lines = text.wrap_text(3);
        assert_eq!(6, lines.len());

        assert_eq!(3, lines[0].ascii_width());
        assert_eq!("Hel", lines[0].render(&text.content));

        assert_eq!(3, lines[1].ascii_width());
        assert_eq!("🧑‍🚀l", lines[1].render(&text.content));

        assert_eq!(3, lines[2].ascii_width());
        assert_eq!("o,w", lines[2].render(&text.content));

        assert_eq!(3, lines[3].ascii_width());
        assert_eq!("o🧑‍🚀", lines[3].render(&text.content));

        assert_eq!(3, lines[4].ascii_width());
        assert_eq!("rld", lines[4].render(&text.content));

        assert_eq!(1, lines[5].ascii_width());
        assert_eq!("!", lines[5].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_wide_width() {
        let text = Text::new("Hel🧑‍🚀lo,wo🧑‍🚀rld!");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(10, lines[0].ascii_width());
        assert_eq!("Hel🧑‍🚀lo,wo", lines[0].render(&text.content));

        assert_eq!(6, lines[1].ascii_width());
        assert_eq!("🧑‍🚀rld!", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_mixed_emoji_newlines() {
        let text = Text::new("Hello🎉\n🚀World\r\n🌟End!");
        let lines = text.wrap_text(8);
        assert_eq!(3, lines.len());

        assert_eq!(7, lines[0].ascii_width());
        assert_eq!("Hello🎉", lines[0].render(&text.content));

        assert_eq!(7, lines[1].ascii_width());
        assert_eq!("🚀World", lines[1].render(&text.content));

        assert_eq!(6, lines[2].ascii_width());
        assert_eq!("🌟End!", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_multiple_emoji_sequence() {
        let text = Text::new("🎯🎨🎪🎭🎮");
        let lines = text.wrap_text(4);
        assert_eq!(3, lines.len());

        assert_eq!(4, lines[0].ascii_width());
        assert_eq!("🎯🎨", lines[0].render(&text.content));

        assert_eq!(4, lines[1].ascii_width());
        assert_eq!("🎪🎭", lines[1].render(&text.content));

        assert_eq!(2, lines[2].ascii_width());
        assert_eq!("🎮", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_with_text_tight() {
        let text = Text::new("Hi🌈there🔥world🚀!");
        let lines = text.wrap_text(6);
        assert_eq!(4, lines.len());

        assert_eq!(6, lines[0].ascii_width());
        assert_eq!("Hi🌈th", lines[0].render(&text.content));

        assert_eq!(6, lines[1].ascii_width());
        assert_eq!("ere🔥w", lines[1].render(&text.content));

        assert_eq!(6, lines[2].ascii_width());
        assert_eq!("orld🚀", lines[2].render(&text.content));

        assert_eq!(1, lines[3].ascii_width());
        assert_eq!("!", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_complex_emoji_text() {
        let text = Text::new("🧑‍🚀👨‍💻👩‍🎨\nCode🚀Fast💨");
        let lines = text.wrap_text(7);
        assert_eq!(3, lines.len());

        assert_eq!(6, lines[0].ascii_width());
        assert_eq!("🧑‍🚀👨‍💻👩‍🎨", lines[0].render(&text.content));

        assert_eq!(7, lines[1].ascii_width());
        assert_eq!("Code🚀F", lines[1].render(&text.content));

        assert_eq!(5, lines[2].ascii_width());
        assert_eq!("ast💨", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_single_emoji_per_line() {
        let text = Text::new("🎉🎊🎈🎁");
        let lines = text.wrap_text(2);
        assert_eq!(4, lines.len());

        assert_eq!(2, lines[0].ascii_width());
        assert_eq!("🎉", lines[0].render(&text.content));

        assert_eq!(2, lines[1].ascii_width());
        assert_eq!("🎊", lines[1].render(&text.content));

        assert_eq!(2, lines[2].ascii_width());
        assert_eq!("🎈", lines[2].render(&text.content));

        assert_eq!(2, lines[3].ascii_width());
        assert_eq!("🎁", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_with_cr_lf() {
        let text = Text::new("Fun🎪\r\nTime⏰\rNow🌟");
        let lines = text.wrap_text(5);
        assert_eq!(4, lines.len());

        assert_eq!(5, lines[0].ascii_width());
        assert_eq!("Fun🎪", lines[0].render(&text.content));

        assert_eq!(4, lines[1].ascii_width());
        assert_eq!("Time", lines[1].render(&text.content));

        assert_eq!(2, lines[2].ascii_width());
        assert_eq!("⏰", lines[2].render(&text.content));

        assert_eq!(5, lines[3].ascii_width());
        assert_eq!("Now🌟", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_long_emoji_text_mix() {
        let text = Text::new("Programming🧑‍💻is🚀awesome🎯today!");
        let lines = text.wrap_text(9);
        assert_eq!(4, lines.len());

        assert_eq!(9, lines[0].ascii_width());
        assert_eq!("Programmi", lines[0].render(&text.content));

        assert_eq!(9, lines[1].ascii_width());
        assert_eq!("ng🧑‍💻is🚀a", lines[1].render(&text.content));

        assert_eq!(9, lines[2].ascii_width());
        assert_eq!("wesome🎯t", lines[2].render(&text.content));

        assert_eq!(5, lines[3].ascii_width());
        assert_eq!("oday!", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_trailing_newlines() {
        let text = Text::new("Hello🌍\n\n🎉Party\n");
        let lines = text.wrap_text(10);
        assert_eq!(4, lines.len());

        assert_eq!(7, lines[0].ascii_width());
        assert_eq!("Hello🌍", lines[0].render(&text.content));

        assert_eq!(0, lines[1].ascii_width());
        assert_eq!("", lines[1].render(&text.content));

        assert_eq!(7, lines[2].ascii_width());
        assert_eq!("🎉Party", lines[2].render(&text.content));

        assert_eq!(0, lines[3].ascii_width());
        assert_eq!("", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_mixed_unicode_emoji() {
        let text = Text::new("你好🌸world🎌こんにちは🗾!");
        let lines = text.wrap_text(8);
        assert_eq!(4, lines.len());

        assert_eq!(8, lines[0].ascii_width());
        assert_eq!("你好🌸wo", lines[0].render(&text.content));

        assert_eq!(7, lines[1].ascii_width());
        assert_eq!("rld🎌こ", lines[1].render(&text.content));

        assert_eq!(8, lines[2].ascii_width());
        assert_eq!("んにちは", lines[2].render(&text.content));

        assert_eq!(3, lines[3].ascii_width());
        assert_eq!("🗾!", lines[3].render(&text.content));
    }
}
