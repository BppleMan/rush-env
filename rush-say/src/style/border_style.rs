use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct BorderStyle {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub size: usize,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::simple()
    }
}

impl Display for BorderStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}{}{}", self.top_left, self.horizontal, self.top_right)?;
        writeln!(f, "{} {}", self.vertical, self.vertical)?;
        writeln!(f, "{}{}{}", self.bottom_left, self.horizontal, self.bottom_right)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! border_factory {
    ($cb:ident) => {
        $cb! {
            simple, "simple", '+', '+', '+', '+', '─', '│', 1;
            single, "single", '┌', '┐', '└', '┘', '─', '│', 1;
            double, "double", '╔', '╗', '╚', '╝', '═', '║', 1;
            rounded, "rounded", '╭', '╮', '╰', '╯', '─', '│', 1;
            heavy, "heavy", '┏', '┓', '┗', '┛', '━', '┃', 1;
            block, "block", '█', '█', '█', '█', '█', '█', 1;
            dotted, "dotted", '.', '.', ':', ':', '.', ':', 1;
            ascii_light, "ascii_light", '/', '\\', '\\', '/', '-', '|', 1;
            angled, "angled", '╱', '╲', '╲', '╱', '─', '│', 1;
            thin_double, "thin_double", '╒', '╕', '╘', '╛', '─', '│', 1;
            square, "square", '■', '■', '■', '■', '■', '■', 1;
            dashed_round, "dashed_round", '●', '●', '●', '●', '•', '•', 1;
            wave, "wave", '≈', '≈', '≈', '≈', '≈', '∣', 1;
            zigzag, "zigzag", '╱', '╲', '╲', '╱', '╌', '╎', 1;
            dotted_box, "dotted_box", '∙', '∙', '∙', '∙', '·', '·', 1;
            triangle_chain, "triangle_chain", '◤', '◥', '◣', '◢', '▲', '▶', 1;
            ascii_flower, "ascii_flower", '*', '*', '*', '*', '~', '|', 1;
        }
    };
}

macro_rules! border_style {
    ($($name:ident, $con:expr, $top_left:expr, $top_right:expr, $bottom_left:expr, $bottom_right:expr, $horizontal:expr, $vertical:expr, $size:expr);+ $(;)?) => {
        impl BorderStyle {
            $(
                pub const fn $name() -> Self {
                    BorderStyle {
                        top_left: $top_left,
                        top_right: $top_right,
                        bottom_left: $bottom_left,
                        bottom_right: $bottom_right,
                        horizontal: $horizontal,
                        vertical: $vertical,
                        size: $size,
                    }
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

#[allow(unused)]
macro_rules! index_of {
    (
        $idx:expr, $n:expr;
    ) => {
        BorderStyle::simple()
    };
    (
        $idx:expr, $n:expr;
        $name:ident $(,$ex:expr)+;
    ) => {
        BorderStyle::$name()
    };
    (
        $idx:expr, $n:expr;
        $name:ident $(,$ex:expr)+;
        $(
            $name_remain:ident $(,$ex_remain:expr)+
        );+ $(;)?
    ) => {
        if $idx == $n {
            return BorderStyle::$name()
        }
        index_of! {
            $idx, $n + 1;
            $($name_remain $(,$ex_remain)+;)*
        }

    };
    (
        $name:ident $(,$ex:expr)+;
        $(
            $name_remain:ident $(,$ex_remain:expr)+
        );+ $(;)?
    ) => {
        |idx: usize| -> BorderStyle {{
            index_of!{
                idx, 0;
                $($name_remain $(,$ex_remain)+;)*
            }
        }}
    };
}

macro_rules! as_array {
    ($($name:ident $(,$ignore:expr)* ;)+) => {
        [ $( BorderStyle::$name() ),+ ]
    };
}

border_factory!(border_style);

const COUNT: usize = border_factory!(count_items);
const BORDER_STYLES: [BorderStyle; COUNT] = border_factory!(as_array);

impl BorderStyle {
    // 方案一：生成式闭包
    // pub fn random(random: usize) -> Self {
    //     border_factory!(index_of)(random % COUNT)
    // }

    // 方案二：常量数组
    pub fn random(random: usize) -> Self {
        BORDER_STYLES[random % COUNT]
    }
}
