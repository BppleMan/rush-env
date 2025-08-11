#[macro_export]
macro_rules! comment_ctor {
    ($name:ident, $prefix:expr) => {
        pub fn $name() -> Self {
            CommentStyle::new($prefix)
        }
    };
    ($name:ident, $prefix:expr, $suffix:expr) => {
        pub fn $name() -> Self {
            CommentStyle::with_suffix($prefix, Some($suffix))
        }
    };
    ($name:ident, $prefix:expr, $open:expr, $close:expr) => {
        pub fn $name() -> Self {
            CommentStyle::new($prefix).set_open($open).set_close($close)
        }
    };
}

#[macro_export]
macro_rules! comment_types {
    ( $( $name:ident, $($args:expr),+ $(,)? );* $(;)? ) => {
        impl CommentStyle {
            $(
                $crate::comment_ctor!($name $(, $args)+);
            )*
        }
    };
}
