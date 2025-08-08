use crate::style::{BorderStyle, CommentStyle};
use crate::widget::{Align, Constraints, Container, Size, Text, Widget};
use std::io::Write;

#[derive(Default, Debug, Clone)]
pub struct Bubble {
    pub inner: Container<Text>,
    pub comment: Option<CommentStyle>,
    pub width: usize,
}

impl Bubble {
    pub fn new(text: impl AsRef<str>) -> Self {
        Bubble {
            inner: Container::new(Text::new(text).set_align(Align::Left))
                .set_border(BorderStyle::rounded())
                .set_padding(5)
                .set_margin(1)
                .set_align(Align::Center),
            comment: None,
            width: 40,
        }
    }

    pub fn say(mut self, writer: &mut impl Write) -> std::io::Result<()> {
        let constraints = Constraints { max_width: self.width };
        self.inner.layout(constraints);
        let size = match &self.comment {
            Some(comment) => Size {
                width: self.inner.size().width + comment.size(),
                height: self.inner.size().height,
            },
            None => self.inner.size(),
        };
        for row in 0..size.height {
            match &self.comment {
                None => self.inner.render(writer, row)?,
                Some(comment) => {
                    write!(writer, "{}", comment.prefix)?;
                    self.inner.render(writer, row)?;
                    if let Some(suffix) = comment.suffix {
                        write!(writer, "{suffix}")?;
                    }
                }
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

impl Bubble {
    pub fn set_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn set_comment(mut self, comment: CommentStyle) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_border(mut self, border: BorderStyle) -> Self {
        self.inner = self.inner.set_border(border);
        self
    }

    pub fn set_padding(mut self, padding: usize) -> Self {
        self.inner = self.inner.set_padding(padding);
        self
    }

    pub fn set_margin(mut self, margin: usize) -> Self {
        self.inner = self.inner.set_margin(margin);
        self
    }

    pub fn set_align(mut self, align: Align) -> Self {
        self.inner = self.inner.set_align(align);
        self
    }

    pub fn set_text_align(mut self, align: Align) -> Self {
        self.inner.inner = self.inner.inner.set_align(align);
        self
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
        let bubble = Bubble::new(text.trim())
            .set_width(50)
            .set_padding(5)
            .set_align(Align::Right)
            .set_text_align(Align::Left);
        bubble.say(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        println!("{output}");
        // assert!(output.contains("Hello, world!"));
        // assert!(output.contains("/* Comment */"));
    }
}
