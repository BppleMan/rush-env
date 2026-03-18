use clap::ValueEnum;
use std::fmt::{self, Display};

#[derive(Default, Debug, Clone, Copy)]
pub struct Constraints {
    pub max_width: usize,
    pub max_height: usize,
}

impl Constraints {
    pub fn new(max_width: usize, max_height: usize) -> Self {
        Self { max_width, max_height }
    }
}

impl Display for Constraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<={}x{}", self.max_width, self.max_height)
    }
}

#[derive(Default, Debug, Copy, Clone, ValueEnum)]
pub enum Axis {
    #[default]
    Vertical,
    Horizontal,
}

// pub fn visual_width_chars(chars: &[char]) -> usize {
//     chars.iter().copied().map(visual_width_char).sum()
// }
//
// /// 视觉宽度换算（可随时自定义规则）
// pub fn visual_width_char(ch: char) -> usize {
//     match ch {
//         '\u{4e00}'..='\u{9fff}'   // CJK汉字
//         | '\u{3000}'..='\u{303f}' // CJK标点
//         | '\u{3040}'..='\u{30ff}' // 日文
//         | '\u{ff00}'..='\u{ffef}' // 全角
//         => 2,
//         '\u{1f300}'..='\u{1f6ff}' | '\u{1f900}'..='\u{1f9ff}' => 2, // emoji
//         _ => 1,
//     }
// }
