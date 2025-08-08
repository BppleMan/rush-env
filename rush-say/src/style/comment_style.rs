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

impl CommentStyle {
    pub fn rust() -> Self {
        CommentStyle::with_suffix("// ", "")
    }

    pub fn rust_doc() -> Self {
        CommentStyle::with_suffix("/// ", "")
    }

    pub fn java() -> Self {
        CommentStyle::with_suffix("// ", "")
    }

    pub fn kotlin() -> Self {
        CommentStyle::with_suffix("// ", "")
    }

    pub fn html() -> Self {
        CommentStyle::with_suffix("<!-- ", " -->")
    }

    pub fn xml() -> Self {
        CommentStyle::with_suffix("<!-- ", " -->")
    }
}
