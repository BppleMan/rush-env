pub mod border {
    use clap::ValueEnum;
    use rush_say::BorderStyle;
    use std::fmt::{Display, Formatter};
    use std::str::FromStr;

    #[derive(Default, Debug, Clone, Copy, ValueEnum)]
    pub enum BorderType {
        #[default]
        Simple,
        Single,
        Double,
        Rounded,
        Heavy,
        Block,
        Dotted,
        AsciiLight,
        Angled,
        ThinDouble,
        Square,
        DashedRound,
        Wave,
        Zigzag,
        DottedBox,
        TriangleChain,
        AsciiFlower,
    }

    impl BorderType {
        pub fn build(&self) -> BorderStyle {
            match self {
                BorderType::Simple => BorderStyle::simple(),
                BorderType::Single => BorderStyle::single(),
                BorderType::Double => BorderStyle::double(),
                BorderType::Rounded => BorderStyle::rounded(),
                BorderType::Heavy => BorderStyle::heavy(),
                BorderType::Block => BorderStyle::block(),
                BorderType::Dotted => BorderStyle::dotted(),
                BorderType::AsciiLight => BorderStyle::ascii_light(),
                BorderType::Angled => BorderStyle::angled(),
                BorderType::ThinDouble => BorderStyle::thin_double(),
                BorderType::Square => BorderStyle::square(),
                BorderType::DashedRound => BorderStyle::dashed_round(),
                BorderType::Wave => BorderStyle::wave(),
                BorderType::Zigzag => BorderStyle::zigzag(),
                BorderType::DottedBox => BorderStyle::dotted_box(),
                BorderType::TriangleChain => BorderStyle::triangle_chain(),
                BorderType::AsciiFlower => BorderStyle::ascii_flower(),
            }
        }
    }

    impl Display for BorderType {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }

    impl FromStr for BorderType {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "simple" => Ok(BorderType::Simple),
                "single" => Ok(BorderType::Single),
                "double" => Ok(BorderType::Double),
                "rounded" => Ok(BorderType::Rounded),
                "heavy" => Ok(BorderType::Heavy),
                "block" => Ok(BorderType::Block),
                "dotted" => Ok(BorderType::Dotted),
                "ascii_light" => Ok(BorderType::AsciiLight),
                "angled" => Ok(BorderType::Angled),
                "thin_double" => Ok(BorderType::ThinDouble),
                "square" => Ok(BorderType::Square),
                "dashed_round" => Ok(BorderType::DashedRound),
                "wave" => Ok(BorderType::Wave),
                "zigzag" => Ok(BorderType::Zigzag),
                "dotted_box" => Ok(BorderType::DottedBox),
                "triangle_chain" => Ok(BorderType::TriangleChain),
                "ascii_flower" => Ok(BorderType::AsciiFlower),
                _ => Err(format!("Unknown border type {s}")),
            }
        }
    }
}

pub mod comment {
    use clap::ValueEnum;
    use rush_say::CommentStyle;
    use std::fmt::{Display, Formatter};
    use std::str::FromStr;

    macro_rules! comment_type {
        ($($name:ident, $prefix:expr, $suffix:expr, $open:expr, $close:expr, $con:expr);+ $(;)?) => {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy, ValueEnum)]
            pub enum CommentType {
                $(
                    $name,
                )+
            }

            impl CommentType {
                pub fn build(&self) -> CommentStyle {
                    match self {
                        $(Self::$name => CommentStyle::$name()),+
                    }
                }
            }

            impl Display for CommentType {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$name => write!(f, "{}", $con),)+
                    }
                }
            }

            impl FromStr for CommentType {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $($con => Ok(Self::$name),)+
                        _ => Err(format!("Unknown comment type {s}")),
                    }
                }
            }
        };
    }

    comment_type! {
        rust, "//", None, None, None, "rust";
        rust_doc, "///", None, None, None, "rust_doc";
        java, "//", None, None, None, "java";
        shell, "#", None, None, None, "shell";
        vimrc, "\"", None, None, None, "vimrc";
        lua, "--", None, None, None, "lua";
        sql, "--", None, None, None, "sql";
        xml, "<!--", Some("-->"), None, None, "xml";
        java_doc, " * ", None, Some("/**"), Some(" */"), "java_doc";
        c_block, "", None, Some("/*"), Some("*/"), "c_block";
        lua_block, "", None, Some("--[["), Some("]]"), "lua_block";
        python_triple_single, "", None, Some("'''"), Some("'''"), "python_triple_single";
        python_triple_double, "", None, Some("\"\"\""), Some("\"\"\""), "python_triple_double";
    }
}
