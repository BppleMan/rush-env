use crate::widget::{Align, Constraints, Size, Widget};
use std::io::Write;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct Text {
    pub content: String,
    pub graphemes: Vec<TextGrapheme>,
    pub align: Align,
    lines: Vec<TextLine>,
    size: Size,
}

#[derive(Default, Debug, Clone)]
pub struct TextGrapheme {
    pub range: Range<usize>,
    pub width: usize,
}

#[derive(Default, Debug, Clone)]
pub struct TextLine {
    pub graphemes: Vec<TextGrapheme>,
    pub width: usize,
}

impl Text {
    pub fn new(content: impl AsRef<str>) -> Text {
        Self::with_align(content, Align::Center)
    }

    pub fn with_align(content: impl AsRef<str>, align: Align) -> Text {
        let content = content.as_ref().to_string();
        let grapheme_indices = content.grapheme_indices(true);
        let graphemes = grapheme_indices
            .map(|(start, grapheme)| {
                let range = start..start + grapheme.len();
                let width = grapheme.width();
                TextGrapheme { range, width }
            })
            .collect::<Vec<_>>();
        Text {
            content,
            graphemes,
            align,
            ..Default::default()
        }
    }

    pub fn set_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn wrap_text(&mut self, max_width: usize) -> Vec<TextLine> {
        let mut lines: Vec<TextLine> = vec![];
        let mut current_line: TextLine = TextLine::default();
        for grapheme in std::mem::take(&mut self.graphemes) {
            let grapheme_str = &self.content[grapheme.range.clone()];
            println!("{grapheme_str:?}");
            match grapheme_str {
                "\n" | "\r" | "\r\n" => {
                    lines.push(std::mem::take(&mut current_line));
                }
                _ => {
                    if current_line.width + grapheme.width > max_width {
                        // 如果当前行宽度超过限制，则换行
                        lines.push(std::mem::take(&mut current_line));
                    }
                    current_line.push(grapheme);
                }
            }
        }
        lines.push(current_line);
        lines
    }
}

impl Widget for Text {
    fn layout(&mut self, constraints: Constraints) -> Size {
        self.lines = self.wrap_text(constraints.wrap_width);
        let width = self
            .lines
            .iter()
            .map(|line| line.width)
            .max()
            .unwrap_or(0)
            .max(constraints.render_width);
        let height = self.lines.len();
        self.size = Size { width, height };
        self.size
    }

    fn render(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        let line = &self.lines[row];
        let text = line.render(&self.content);
        let pad = self.size.width.saturating_sub(line.width);
        let (left, right) = match self.align {
            Align::Left => (0, pad),
            Align::Right => (pad, 0),
            Align::Center => (pad / 2, pad - (pad / 2)),
        };
        write!(writer, "{}{}{}", " ".repeat(left), text, " ".repeat(right))
    }
}

impl TextGrapheme {
    pub fn render<'a>(&'a self, content: &'a str) -> &'a str {
        &content[self.range.clone()]
    }
}

impl TextLine {
    pub fn push(&mut self, grapheme: TextGrapheme) {
        self.width += grapheme.width;
        self.graphemes.push(grapheme);
    }

    pub fn render<'a>(&'a self, content: &'a str) -> &'a str {
        let range = match (self.graphemes.first(), self.graphemes.last()) {
            (Some(first), Some(last)) => first.range.start..last.range.end,
            _ => 0..0, // 如果没有字符，则返回空范围
        };
        &content[range]
    }
}

#[cfg(test)]
mod tests {
    use crate::widget::text::Text;

    #[test]
    fn test_text_widget() {
        let mut text = Text::new("Hel🧑‍🚀lo,\r\nworld!");
        let lines = text.wrap_text(4);
        assert_eq!(5, lines.len());

        assert_eq!(3, lines[0].width);
        assert_eq!("Hel", lines[0].render(&text.content));

        assert_eq!(4, lines[1].width);
        assert_eq!("🧑‍🚀lo", lines[1].render(&text.content));

        assert_eq!(1, lines[2].width);
        assert_eq!(",", lines[2].render(&text.content));

        assert_eq!(4, lines[3].width);
        assert_eq!("worl", lines[3].render(&text.content));

        assert_eq!(2, lines[4].width);
        assert_eq!("d!", lines[4].render(&text.content));
    }

    #[test]
    fn test_text_widget_simple_ascii() {
        let mut text = Text::new("hello world");
        let lines = text.wrap_text(5);
        assert_eq!(3, lines.len());

        assert_eq!(5, lines[0].width);
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(5, lines[1].width);
        assert_eq!(" worl", lines[1].render(&text.content));

        assert_eq!(1, lines[2].width);
        assert_eq!("d", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_exact_fit() {
        let mut text = Text::new("hello");
        let lines = text.wrap_text(5);
        assert_eq!(1, lines.len());

        assert_eq!(5, lines[0].width);
        assert_eq!("hello", lines[0].render(&text.content));
    }

    #[test]
    fn test_text_widget_single_char_per_line() {
        let mut text = Text::new("abc");
        let lines = text.wrap_text(1);
        assert_eq!(3, lines.len());

        assert_eq!(1, lines[0].width);
        assert_eq!("a", lines[0].render(&text.content));

        assert_eq!(1, lines[1].width);
        assert_eq!("b", lines[1].render(&text.content));

        assert_eq!(1, lines[2].width);
        assert_eq!("c", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_empty_string() {
        let mut text = Text::new("");
        let lines = text.wrap_text(10);
        assert_eq!(1, lines.len());

        assert_eq!(0, lines[0].width);
        assert_eq!("", lines[0].render(&text.content));
    }

    #[test]
    fn test_text_widget_newline_only() {
        let mut text = Text::new("hello\nworld");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].width);
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(5, lines[1].width);
        assert_eq!("world", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_multiple_newlines() {
        let mut text = Text::new("a\n\nb");
        let lines = text.wrap_text(10);
        assert_eq!(3, lines.len());

        assert_eq!(1, lines[0].width);
        assert_eq!("a", lines[0].render(&text.content));

        assert_eq!(0, lines[1].width);
        assert_eq!("", lines[1].render(&text.content));

        assert_eq!(1, lines[2].width);
        assert_eq!("b", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_unicode_wide_chars() {
        let mut text = Text::new("你好世界");
        let lines = text.wrap_text(4);
        assert_eq!(2, lines.len());

        assert_eq!(4, lines[0].width);
        assert_eq!("你好", lines[0].render(&text.content));

        assert_eq!(4, lines[1].width);
        assert_eq!("世界", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_mixed_ascii_unicode() {
        let mut text = Text::new("hello你好");
        let lines = text.wrap_text(6);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].width);
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(4, lines[1].width);
        assert_eq!("你好", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_cr_line_ending() {
        let mut text = Text::new("line1\rline2");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].width);
        assert_eq!("line1", lines[0].render(&text.content));

        assert_eq!(5, lines[1].width);
        assert_eq!("line2", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_trailing_newline() {
        let mut text = Text::new("hello\n");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(5, lines[0].width);
        assert_eq!("hello", lines[0].render(&text.content));

        assert_eq!(0, lines[1].width);
        assert_eq!("", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_very_long_word() {
        let mut text = Text::new("supercalifragilisticexpialidocious");
        let lines = text.wrap_text(10);
        assert_eq!(4, lines.len());

        assert_eq!(10, lines[0].width);
        assert_eq!("supercalif", lines[0].render(&text.content));

        assert_eq!(10, lines[1].width);
        assert_eq!("ragilistic", lines[1].render(&text.content));

        assert_eq!(10, lines[2].width);
        assert_eq!("expialidoc", lines[2].render(&text.content));

        assert_eq!(4, lines[3].width);
        assert_eq!("ious", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_spaces_and_wrapping() {
        let mut text = Text::new("a  b  c");
        let lines = text.wrap_text(3);
        assert_eq!(3, lines.len());

        assert_eq!(3, lines[0].width);
        assert_eq!("a  ", lines[0].render(&text.content));

        assert_eq!(3, lines[1].width);
        assert_eq!("b  ", lines[1].render(&text.content));

        assert_eq!(1, lines[2].width);
        assert_eq!("c", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_variant_narrow() {
        let mut text = Text::new("Hel🧑‍🚀lo,wo🧑‍🚀rld!");
        let lines = text.wrap_text(3);
        assert_eq!(6, lines.len());

        assert_eq!(3, lines[0].width);
        assert_eq!("Hel", lines[0].render(&text.content));

        assert_eq!(3, lines[1].width);
        assert_eq!("🧑‍🚀l", lines[1].render(&text.content));

        assert_eq!(3, lines[2].width);
        assert_eq!("o,w", lines[2].render(&text.content));

        assert_eq!(3, lines[3].width);
        assert_eq!("o🧑‍🚀", lines[3].render(&text.content));

        assert_eq!(3, lines[4].width);
        assert_eq!("rld", lines[4].render(&text.content));

        assert_eq!(1, lines[5].width);
        assert_eq!("!", lines[5].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_variant_wide() {
        let mut text = Text::new("Hel🧑‍🚀lo,wo🧑‍🚀rld!");
        let lines = text.wrap_text(10);
        assert_eq!(2, lines.len());

        assert_eq!(10, lines[0].width);
        assert_eq!("Hel🧑‍🚀lo,wo", lines[0].render(&text.content));

        assert_eq!(6, lines[1].width);
        assert_eq!("🧑‍🚀rld!", lines[1].render(&text.content));
    }

    #[test]
    fn test_text_widget_mixed_emoji_newlines() {
        let mut text = Text::new("Hello🎉\n🚀World\r\n🌟End!");
        let lines = text.wrap_text(8);
        assert_eq!(3, lines.len());

        assert_eq!(7, lines[0].width);
        assert_eq!("Hello🎉", lines[0].render(&text.content));

        assert_eq!(7, lines[1].width);
        assert_eq!("🚀World", lines[1].render(&text.content));

        assert_eq!(6, lines[2].width);
        assert_eq!("🌟End!", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_multiple_emoji_sequence() {
        let mut text = Text::new("🎯🎨🎪🎭🎮");
        let lines = text.wrap_text(4);
        assert_eq!(3, lines.len());

        assert_eq!(4, lines[0].width);
        assert_eq!("🎯🎨", lines[0].render(&text.content));

        assert_eq!(4, lines[1].width);
        assert_eq!("🎪🎭", lines[1].render(&text.content));

        assert_eq!(2, lines[2].width);
        assert_eq!("🎮", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_with_text_tight() {
        let mut text = Text::new("Hi🌈there🔥world🚀!");
        let lines = text.wrap_text(6);
        assert_eq!(4, lines.len());

        assert_eq!(6, lines[0].width);
        assert_eq!("Hi🌈th", lines[0].render(&text.content));

        assert_eq!(6, lines[1].width);
        assert_eq!("ere🔥w", lines[1].render(&text.content));

        assert_eq!(6, lines[2].width);
        assert_eq!("orld🚀", lines[2].render(&text.content));

        assert_eq!(1, lines[3].width);
        assert_eq!("!", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_complex_emoji_text() {
        let mut text = Text::new("🧑‍🚀👨‍💻👩‍🎨\nCode🚀Fast💨");
        let lines = text.wrap_text(7);
        assert_eq!(3, lines.len());

        assert_eq!(6, lines[0].width);
        assert_eq!("🧑‍🚀👨‍💻👩‍🎨", lines[0].render(&text.content));

        assert_eq!(7, lines[1].width);
        assert_eq!("Code🚀F", lines[1].render(&text.content));

        assert_eq!(5, lines[2].width);
        assert_eq!("ast💨", lines[2].render(&text.content));
    }

    #[test]
    fn test_text_widget_single_emoji_per_line() {
        let mut text = Text::new("🎉🎊🎈🎁");
        let lines = text.wrap_text(2);
        assert_eq!(4, lines.len());

        assert_eq!(2, lines[0].width);
        assert_eq!("🎉", lines[0].render(&text.content));

        assert_eq!(2, lines[1].width);
        assert_eq!("🎊", lines[1].render(&text.content));

        assert_eq!(2, lines[2].width);
        assert_eq!("🎈", lines[2].render(&text.content));

        assert_eq!(2, lines[3].width);
        assert_eq!("🎁", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_with_cr_lf() {
        let mut text = Text::new("Fun🎪\r\nTime⏰\rNow🌟");
        let lines = text.wrap_text(5);
        assert_eq!(4, lines.len());

        assert_eq!(5, lines[0].width);
        assert_eq!("Fun🎪", lines[0].render(&text.content));

        assert_eq!(4, lines[1].width);
        assert_eq!("Time", lines[1].render(&text.content));

        assert_eq!(2, lines[2].width);
        assert_eq!("⏰", lines[2].render(&text.content));

        assert_eq!(5, lines[3].width);
        assert_eq!("Now🌟", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_long_emoji_text_mix() {
        let mut text = Text::new("Programming🧑‍💻is🚀awesome🎯today!");
        let lines = text.wrap_text(9);
        assert_eq!(4, lines.len());

        assert_eq!(9, lines[0].width);
        assert_eq!("Programmi", lines[0].render(&text.content));

        assert_eq!(9, lines[1].width);
        assert_eq!("ng🧑‍💻is🚀a", lines[1].render(&text.content));

        assert_eq!(9, lines[2].width);
        assert_eq!("wesome🎯t", lines[2].render(&text.content));

        assert_eq!(5, lines[3].width);
        assert_eq!("oday!", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_emoji_trailing_newlines() {
        let mut text = Text::new("Hello🌍\n\n🎉Party\n");
        let lines = text.wrap_text(10);
        assert_eq!(4, lines.len());

        assert_eq!(7, lines[0].width);
        assert_eq!("Hello🌍", lines[0].render(&text.content));

        assert_eq!(0, lines[1].width);
        assert_eq!("", lines[1].render(&text.content));

        assert_eq!(7, lines[2].width);
        assert_eq!("🎉Party", lines[2].render(&text.content));

        assert_eq!(0, lines[3].width);
        assert_eq!("", lines[3].render(&text.content));
    }

    #[test]
    fn test_text_widget_mixed_unicode_emoji() {
        let mut text = Text::new("你好🌸world🎌こんにちは🗾!");
        let lines = text.wrap_text(8);
        assert_eq!(4, lines.len());

        assert_eq!(8, lines[0].width);
        assert_eq!("你好🌸wo", lines[0].render(&text.content));

        assert_eq!(7, lines[1].width);
        assert_eq!("rld🎌こ", lines[1].render(&text.content));

        assert_eq!(8, lines[2].width);
        assert_eq!("んにちは", lines[2].render(&text.content));

        assert_eq!(3, lines[3].width);
        assert_eq!("🗾!", lines[3].render(&text.content));
    }
}
