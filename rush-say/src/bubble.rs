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
    pub fn new(
        text: impl AsRef<str>,
        width: usize,
        padding: usize,
        margin: usize,
        border: BorderStyle,
        comment: Option<CommentStyle>,
    ) -> Self {
        Bubble {
            inner: Container::new(Text::new(text))
                .set_border(border)
                .set_padding(padding)
                .set_margin(margin)
                .set_align(Align::Center),
            comment,
            width,
        }
    }

    pub fn say(mut self, writer: &mut impl Write) -> std::io::Result<()> {
        let constraints = Constraints {
            render_width: self.width,
            wrap_width: self.width,
        };
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
                        writeln!(writer, "{}", suffix)?;
                    } else {
                        writeln!(writer)?;
                    }
                }
            }
        }
        Ok(())
    }
}
