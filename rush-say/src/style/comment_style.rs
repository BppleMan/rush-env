use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct CommentStyle {
    pub prefix: &'static str,
    pub suffix: Option<&'static str>,
    size: usize,
}

impl Default for CommentStyle {
    fn default() -> Self {
        CommentStyle::new("#", None)
    }
}

impl CommentStyle {
    pub fn new(prefix: &'static str, suffix: Option<&'static str>) -> Self {
        let size = prefix.width() + suffix.map_or(0, |s| s.width());
        CommentStyle { prefix, suffix, size }
    }

    pub fn with_suffix(prefix: &'static str, suffix: &'static str) -> Self {
        CommentStyle::new(prefix, Some(suffix))
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
