use crate::border::CommentStyle;
use crate::widget::{Align, Constraints, Container, Size, Text, Widget};
use std::io::Write;

#[derive(Default, Debug, Clone)]
pub struct Bubble {
    pub inner: Container<Text>,
    pub comment: Option<CommentStyle>,
}

impl Bubble {
    // pub fn new(width: usize, padding: usize, border_style: BorderStyle, comment_style: Option<CommentStyle>) -> Self {
    //     Bubble {
    //         style: BubbleStyle { width, padding, border_style },
    //         comment_style,
    //     }
    // }

    pub fn say(&self, writer: &mut impl std::io::Write, content: impl AsRef<str>) -> std::io::Result<()> {
        todo!()
    }
}

impl Widget for Bubble {
    fn layout(&mut self, constraints: Constraints) -> Size {
        todo!()
    }

    fn render(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        todo!()
    }
}
