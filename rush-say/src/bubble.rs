use crate::style::{BorderStyle, CommentStyle};
use crate::widget::{Align, Constraints, Container, Text, Widget};

pub struct Bubble<'a, W>
where
    W: std::io::Write,
{
    pub writer: &'a mut W,
    pub text_align: Align,
    pub border: BorderStyle,
    pub padding: usize,
    pub margin: usize,
    pub align: Align,
    pub comment: Option<CommentStyle>,
    pub width: usize,
}

impl<'a, W> Bubble<'a, W>
where
    W: std::io::Write,
{
    pub fn new(writer: &'a mut W) -> Self {
        Bubble {
            writer,
            text_align: Align::Left,
            border: BorderStyle::rounded(),
            padding: 5,
            margin: 1,
            align: Align::Center,
            comment: None,
            width: 40,
        }
    }

    pub fn example(writer: &'a mut W) -> Self {
        Self {
            writer,
            text_align: Align::Left,
            border: BorderStyle::rounded(),
            padding: 4,
            margin: 1,
            align: Align::Center,
            comment: Some(CommentStyle::shell()),
            width: 88,
        }
    }

    pub fn shell(writer: &'a mut W) -> Self {
        Self {
            writer,
            text_align: Align::Left,
            border: BorderStyle::rounded(),
            padding: 4,
            margin: 1,
            align: Align::Center,
            comment: Some(CommentStyle::shell()),
            width: 40,
        }
    }

    pub fn say(&mut self, text: impl AsRef<str>) -> std::io::Result<()> {
        let mut container = Container::new(Text::new(text).set_align(self.text_align))
            .set_border(self.border)
            .set_padding(self.padding)
            .set_margin(self.margin)
            .set_align(self.align);
        let constraints = Constraints { max_width: self.width };
        container.layout(constraints);
        if let Some(comment_open) = self.comment.as_ref().and_then(|c| c.open) {
            writeln!(self.writer, "{comment_open}")?;
        }
        for row in 0..container.size().height {
            if let Some(comment_prefix) = self.comment.as_ref().map(|c| c.prefix) {
                write!(self.writer, "{comment_prefix}")?;
            }
            container.render(self.writer, row)?;
            if let Some(comment_suffix) = self.comment.as_ref().and_then(|c| c.suffix) {
                write!(self.writer, "{comment_suffix}")?;
            }
            writeln!(self.writer)?;
        }
        if let Some(comment_close) = self.comment.as_ref().and_then(|c| c.close) {
            writeln!(self.writer, "{comment_close}")?;
        }
        Ok(())
    }
}

impl<'a, W> Bubble<'a, W>
where
    W: std::io::Write,
{
    pub fn set_width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }

    pub fn set_comment(&mut self, comment: CommentStyle) -> &mut Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_border(&mut self, border: BorderStyle) -> &mut Self {
        self.border = border;
        self
    }

    pub fn set_padding(&mut self, padding: usize) -> &mut Self {
        self.padding = padding;
        self
    }

    pub fn set_margin(&mut self, margin: usize) -> &mut Self {
        self.margin = margin;
        self
    }

    pub fn set_align(&mut self, align: Align) -> &mut Self {
        self.align = align;
        self
    }

    pub fn set_text_align(&mut self, align: Align) -> &mut Self {
        self.text_align = align;
        self
    }

    pub fn random_style(&mut self, random: usize) -> &mut Self {
        self.set_border(BorderStyle::random(random))
            .set_comment(CommentStyle::random(random))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubble_render() {
        let text = r#"
- 🦀 由 Rust 编写，性能优异
- 💬 类似 cowsay，将文本以气泡对话框形式输出
- 🎨 支持自定义边框、注释风格和对齐方式
- 🛠️ 适合在文本展示有趣消息或提示
- 🧩 易于集成和扩展
"#;
        let mut buffer = Vec::new();
        let mut bubble = Bubble {
            width: 50,
            padding: 0,
            align: Align::Center,
            text_align: Align::Left,
            comment: Some(CommentStyle::java_doc()),
            ..Bubble::new(&mut buffer)
        };
        bubble.say(text).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        println!("{output}");
    }
}
