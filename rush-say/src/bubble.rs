use crate::style::{BorderStyle, CommentStyle};
use crate::widget::{Align, Constraints, Container, Text, Widget};
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

    pub fn example() -> Self {
        let description = r#"
- 📄 为复杂配置文件提供清晰的注释气泡框，增强可读性和理解性
- 📝 将注释文本转换为结构化气泡对话框，使大型配置文件更易维护
- 🖌️ 支持多种边框样式自定义（圆角、方形等），适应不同配置文件的风格需求
- 💬 提供多种注释风格（如Java文档风格），兼容各类编程语言和配置格式
- 📏 灵活的排版控制系统：
    * 可调整文本宽度，适应不同显示环境
    * 自定义内边距(padding)和外边距(margin)
    * 多种对齐方式（左对齐、居中等）
- 🧰 可输出到任何实现了Write trait的目标，便于集成到各种配置生成工具
- 🔄 支持链式调用API（如.set_width().set_border()），简化使用流程
"#;
        Bubble::new(description.trim())
            .set_width(88)
            .set_padding(4)
            .set_border(BorderStyle::rounded())
            .set_comment(CommentStyle::shell())
            .set_align(Align::Center)
            .set_text_align(Align::Left)
    }

    pub fn say(mut self, writer: &mut impl Write) -> std::io::Result<()> {
        let constraints = Constraints { max_width: self.width };
        self.inner.layout(constraints);
        if let Some(comment_open) = self.comment.as_ref().and_then(|c| c.open) {
            writeln!(writer, "{comment_open}")?;
        }
        for row in 0..self.inner.size().height {
            if let Some(comment_prefix) = self.comment.as_ref().map(|c| c.prefix) {
                write!(writer, "{comment_prefix}")?;
            }
            self.inner.render(writer, row)?;
            if let Some(comment_suffix) = self.comment.as_ref().and_then(|c| c.suffix) {
                write!(writer, "{comment_suffix}")?;
            }
            writeln!(writer)?;
        }
        if let Some(comment_close) = self.comment.as_ref().and_then(|c| c.close) {
            writeln!(writer, "{comment_close}")?;
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

    pub fn random_style(self, random: usize) -> Self {
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
        let bubble = Bubble::new(text.trim())
            .set_width(50)
            .set_padding(0)
            .set_align(Align::Center)
            .set_text_align(Align::Left)
            .set_comment(CommentStyle::java_doc());
        bubble.say(&mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        println!("{output}");
    }
}
