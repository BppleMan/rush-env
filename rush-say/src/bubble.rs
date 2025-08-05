use crate::border::{BorderStyle, BubbleStyle, CommentStyle};
use crate::layout::Align;

#[derive(Default, Debug, Clone)]
pub struct Bubble {
    pub style: BubbleStyle,
    pub comment_style: Option<CommentStyle>,
    pub align: Align,
}

impl Bubble {
    // pub fn new(width: usize, padding: usize, border_style: BorderStyle, comment_style: Option<CommentStyle>) -> Self {
    //     Bubble {
    //         style: BubbleStyle { width, padding, border_style },
    //         comment_style,
    //     }
    // }

    pub fn builder() -> SectionBuilder {
        SectionBuilder::default()
    }

    pub fn say(&self, writer: &mut impl std::io::Write, content: impl AsRef<str>) -> std::io::Result<()> {
        let max_line_width = self.style.width - 2 - self.style.padding * 2;
        let top_border = format!(
            "{}{}{}",
            self.style.border_style.top_left,
            self.style.border_style.horizontal.to_string().repeat(self.style.width - 2),
            self.style.border_style.top_right
        );
        writeln!(writer, "{top_border}")?;

        let mut chars = content.as_ref().chars().peekable();
        loop {
            while chars.peek().is_some() && *chars.peek().unwrap() == '\n' {
                write!(writer, "{}", self.style.border_style.vertical)?;
                for _ in 0..(self.style.width - 2) {
                    write!(writer, "{}", self.style.border_style.vertical)?;
                }
                writeln!(writer, "{}", self.style.border_style.vertical)?;
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
            let left = self.style.padding + spaces / 2;
            let right = self.style.padding + (spaces - spaces / 2);
            write!(writer, "{}", self.style.border_style.vertical)?;
            for _ in 0..left {
                write!(writer, " ")?;
            }
            write!(writer, "{current}")?;
            for _ in 0..right {
                write!(writer, " ")?;
            }
            writeln!(writer, "{}", self.style.border_style.vertical)?;
        }
        let bottom_border = format!(
            "{}{}{}",
            self.style.border_style.bottom_left,
            self.style.border_style.horizontal.to_string().repeat(self.style.width - 2),
            self.style.border_style.bottom_right
        );
        writeln!(writer, "{bottom_border}")?;
        Ok(())
    }

    pub fn style(&self) -> &BubbleStyle {
        &self.style
    }
    pub fn comment_style(&self) -> Option<&CommentStyle> {
        self.comment_style.as_ref()
    }
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

#[derive(Debug, Clone)]
pub struct SectionBuilder {
    width: usize,
    padding: usize,
    border_style: BorderStyle,
    comment_style: Option<CommentStyle>,
}

impl Default for SectionBuilder {
    fn default() -> Self {
        SectionBuilder {
            width: 48,
            padding: 2,
            border_style: BorderStyle::default(),
            comment_style: None,
        }
    }
}

impl SectionBuilder {
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }
    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }
    pub fn border_style(mut self, border_style: BorderStyle) -> Self {
        self.border_style = border_style;
        self
    }
    pub fn comment_style(mut self, comment_style: CommentStyle) -> Self {
        self.comment_style = Some(comment_style);
        self
    }
    pub fn build(self) -> Bubble {
        Bubble {
            style: BubbleStyle {
                width: self.width,
                padding: self.padding,
                border_style: self.border_style,
            },
            comment_style: self.comment_style,
        }
    }
}
