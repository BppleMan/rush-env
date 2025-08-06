use crate::border::CommentStyle;
use crate::layout::Align;
use crate::{BorderStyle, Bubble, BubbleStyle};

#[derive(Debug, Clone)]
pub struct BubbleBuilder {
    width: usize,
    padding: usize,
    border_style: BorderStyle,
    comment_style: Option<CommentStyle>,
    align: Align,
}

impl Default for BubbleBuilder {
    fn default() -> Self {
        BubbleBuilder {
            width: 40,
            padding: 2,
            border_style: BorderStyle::default(),
            comment_style: Some(CommentStyle::default()),
            align: Align::Center,
        }
    }
}

impl BubbleBuilder {
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
            align: self.align,
        }
    }
}
