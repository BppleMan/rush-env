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
    use std::fmt::{Display, Formatter};
    use std::str::FromStr;

    #[derive(Default, Debug, Clone, Copy, ValueEnum)]
    pub enum CommentType {
        #[default]
        Rust,
        RustDoc,
        Java,
        Shell,
        VimRc,
        Lua,
        SQL,
        Xml,
        JavaDoc,
        CBlock,
        LuaBlock,
        PythonTripleSingle,
        PythonTripleDouble,
    }

    impl Display for CommentType {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                CommentType::Rust => write!(f, "rust"),
                CommentType::RustDoc => write!(f, "rust_doc"),
                CommentType::Java => write!(f, "java"),
                CommentType::Shell => write!(f, "shell"),
                CommentType::VimRc => write!(f, "vimrc"),
                CommentType::Lua => write!(f, "lua"),
                CommentType::SQL => write!(f, "sql"),
                CommentType::Xml => write!(f, "xml"),
                CommentType::JavaDoc => write!(f, "java_doc"),
                CommentType::CBlock => write!(f, "c_block"),
                CommentType::LuaBlock => write!(f, "lua_block"),
                CommentType::PythonTripleSingle => write!(f, "python_triple_single"),
                CommentType::PythonTripleDouble => write!(f, "python_triple_double"),
            }
        }
    }

    impl FromStr for CommentType {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "rust" => Ok(CommentType::Rust),
                "rust_doc" => Ok(CommentType::RustDoc),
                "java" => Ok(CommentType::Java),
                "shell" => Ok(CommentType::Shell),
                "vimrc" => Ok(CommentType::VimRc),
                "lua" => Ok(CommentType::Lua),
                "sql" => Ok(CommentType::SQL),
                "xml" => Ok(CommentType::Xml),
                "java_doc" => Ok(CommentType::JavaDoc),
                "c_block" => Ok(CommentType::CBlock),
                "lua_block" => Ok(CommentType::LuaBlock),
                "python_triple_single" => Ok(CommentType::PythonTripleSingle),
                "python_triple_double" => Ok(CommentType::PythonTripleDouble),
                _ => Err(format!("Unknown comment type {s}")),
            }
        }
    }

    impl CommentType {
        pub fn build(&self) -> rush_say::CommentStyle {
            match self {
                CommentType::Rust => rush_say::CommentStyle::rust(),
                CommentType::RustDoc => rush_say::CommentStyle::rust_doc(),
                CommentType::Java => rush_say::CommentStyle::java(),
                CommentType::Shell => rush_say::CommentStyle::shell(),
                CommentType::VimRc => rush_say::CommentStyle::vimrc(),
                CommentType::Lua => rush_say::CommentStyle::lua(),
                CommentType::SQL => rush_say::CommentStyle::sql(),
                CommentType::Xml => rush_say::CommentStyle::xml(),
                CommentType::JavaDoc => rush_say::CommentStyle::java_doc(),
                CommentType::CBlock => rush_say::CommentStyle::c_block(),
                CommentType::LuaBlock => rush_say::CommentStyle::lua_block(),
                CommentType::PythonTripleSingle => rush_say::CommentStyle::python_triple_single(),
                CommentType::PythonTripleDouble => rush_say::CommentStyle::python_triple_double(),
            }
        }
    }
}
