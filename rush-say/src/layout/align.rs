use crate::layout::{Pos, Size};
use clap::ValueEnum;

#[derive(Default, Debug, Copy, Clone, ValueEnum)]
pub enum Align {
    TopLeft,
    TopCenter,
    TopRight,

    Left,
    #[default]
    Center,
    Right,

    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Align {
    pub fn calc_relative_pos(&self, parent: &Size, child: &Size) -> Pos {
        match self {
            Align::TopLeft => parent.top_left_of(child),
            Align::TopCenter => parent.top_center_of(child),
            Align::TopRight => parent.top_right_of(child),
            Align::Left => parent.left_of(child),
            Align::Center => parent.center_of(child),
            Align::Right => parent.right_of(child),
            Align::BottomLeft => parent.bottom_left_of(child),
            Align::BottomCenter => parent.bottom_center_of(child),
            Align::BottomRight => parent.bottom_right_of(child),
        }
    }
}
