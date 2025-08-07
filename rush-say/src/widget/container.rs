pub struct Container {}

/// 边框样式定义
#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl Default for crate::BorderStyle {
    fn default() -> Self {
        crate::BorderStyle {
            top_left: ' ',
            top_right: ' ',
            bottom_left: ' ',
            bottom_right: ' ',
            horizontal: '-',
            vertical: ' ',
        }
    }
}
