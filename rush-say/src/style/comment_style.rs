use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy)]
pub struct CommentStyle {
    pub prefix: &'static str,
    pub suffix: Option<&'static str>,
    pub open: Option<&'static str>,
    pub close: Option<&'static str>,
}

impl Default for CommentStyle {
    fn default() -> Self {
        CommentStyle::shell()
    }
}

impl CommentStyle {
    pub const fn new(prefix: &'static str, suffix: Option<&'static str>, open: Option<&'static str>, close: Option<&'static str>) -> Self {
        Self {
            prefix,
            suffix,
            open,
            close,
        }
    }

    pub fn size(&self) -> usize {
        (self.prefix.width() + self.suffix.map_or(0, |s| s.width()))
            .max(self.open.map_or(0, |s| s.width()))
            .max(self.close.map_or(0, |s| s.width()))
    }

    pub fn set_open(mut self, open: &'static str) -> Self {
        self.open = Some(open);
        self
    }

    pub fn set_close(mut self, close: &'static str) -> Self {
        self.close = Some(close);
        self
    }
}

#[rustfmt::skip]
#[macro_export]
macro_rules! comment_factory {
    ($cb:ident) => {
        $cb! {
            rust, "rust", "//", None, None, None;
            rust_doc, "rust_doc", "///", None, None, None;
            java, "java", "//", None, None, None;
            shell, "shell", "#", None, None, None;
            vimrc, "vimrc", "\"", None, None, None;
            lua, "lua", "--", None, None, None;
            sql, "sql", "--", None, None, None;
            xml, "xml", "<!--", Some("-->"), None, None;
            java_doc, "java_doc", " * ", None, Some("/**"), Some(" */");
            c_block, "c_block", "", None, Some("/*"), Some("*/");
            lua_block, "lua_block", "", None, Some("--[["), Some("]]");
            echo, "echo", "echo \"", Some("\""), None, None;
            python_triple_single, "python_triple_single", "", None, Some("'''"), Some("'''");
            python_triple_double, "python_triple_double", "", None, Some("\"\"\""), Some("\"\"\"");
            rust_info, "rust_info", "info!(\"", Some("\");"), None, None;
        }
    };
}

macro_rules! comment_style {
    ($($name:ident, $con:expr, $prefix:expr, $suffix:expr, $open:expr, $close:expr);+ $(;)?) => {
        impl CommentStyle {
            $(
                pub const fn $name() -> Self {
                    CommentStyle::new($prefix, $suffix, $open, $close)
                }
            )+
        }
    };
}

macro_rules! count_items {
    () => { 0 };
    (
        $name:ident $(,$ex:expr)+;
        $(
            $name_remain:ident $(,$ex_remain:expr)+
        );* $(;)?
    ) => {
        1 + count_items!($($name_remain $(, $ex_remain)+;)*)
    };
}

macro_rules! as_array {
    ($($name:ident $(,$ignore:expr)* ;)+) => {
        [ $( CommentStyle::$name() ),+ ]
    };
}

comment_factory!(comment_style);

const COUNT: usize = comment_factory!(count_items);
const COMMENT_STYLES: [CommentStyle; COUNT] = comment_factory!(as_array);

impl CommentStyle {
    pub fn random(random: usize) -> Self {
        COMMENT_STYLES[random % COUNT]
    }
}
