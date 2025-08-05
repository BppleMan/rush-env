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

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle {
            top_left: ' ',
            top_right: ' ',
            bottom_left: ' ',
            bottom_right: ' ',
            horizontal: '-',
            vertical: ' ',
        }
    }
}

/// 气泡样式定义
#[derive(Debug, Clone)]
pub struct BubbleStyle {
    pub width: usize,
    pub padding: usize,
    pub border_style: BorderStyle,
}

impl Default for BubbleStyle {
    fn default() -> Self {
        BubbleStyle {
            width: 48,
            padding: 2,
            border_style: BorderStyle::default(),
        }
    }
}

/// 注释样式定义
#[derive(Debug, Clone)]
pub struct CommentStyle {
    pub prefix: &'static str,
    pub suffix: Option<&'static str>,
}

impl Default for CommentStyle {
    fn default() -> Self {
        CommentStyle { prefix: "#", suffix: None }
    }
}
