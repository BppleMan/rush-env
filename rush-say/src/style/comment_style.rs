use crate::comment_types;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct CommentStyle {
    pub prefix: &'static str,
    pub suffix: Option<&'static str>,
    pub open: Option<&'static str>,
    pub close: Option<&'static str>,
    size: usize,
}

impl Default for CommentStyle {
    fn default() -> Self {
        // CommentStyle::shell()
        todo!()
    }
}

impl CommentStyle {
    pub fn new(prefix: &'static str, suffix: Option<&'static str>, open: Option<&'static str>, close: Option<&'static str>) -> Self {
        Self {
            prefix,
            suffix,
            open,
            close,
            size: prefix.width() + suffix.map_or(0, |s| s.width()),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn set_open(mut self, open: &'static str) -> Self {
        self.open = Some(open);
        self.size = self.size.max(open.width());
        self
    }

    pub fn set_close(mut self, close: &'static str) -> Self {
        self.close = Some(close);
        self.size = self.size.max(close.width());
        self
    }
}

#[macro_export]
macro_rules! comment_catalog {
    ($mac:ident) => {
        $mac!(rust, "//", None, None, None, "rush");
        $mac!(rust_doc, "///", None, None, None, "rush_doc");
        $mac!(java, "//", None, None, None, "java");
        $mac!(shell, "#", None, None, None, "shell");
        $mac!(vimrc, "\"", None, None, None, "vimrc");
        $mac!(lua, "--", None, None, None, "lua");
        $mac!(sql, "--", None, None, None, "sql");
        $mac!(xml, "<!--", Some("-->"), None, None, "xml");
        $mac!(java_doc, " * ", None, Some("/**"), Some(" */"), "java_doc");
        $mac!(c_block, "", None, Some("/*"), Some("*/"), "c_block");
        $mac!(lua_block, "", None, Some("--[["), Some("]]"), "lua_block");
        $mac!(
            python_triple_single,
            "",
            None,
            Some("'''"),
            Some("'''"),
            "python_triple_single"
        );
        $mac!(
            python_triple_double,
            "",
            None,
            Some("\"\"\""),
            Some("\"\"\""),
            "python_triple_double"
        );
    };
}

macro_rules! comment_ctor {
    ($name:ident, $prefix:expr, $suffix:expr, $open:expr, $close:expr, $con:expr) => {
        pub fn $name() -> Self {
            CommentStyle::new($prefix, $suffix, $open, $close)
        }
    };
}

impl CommentStyle {
    comment_catalog!(comment_ctor);
}
