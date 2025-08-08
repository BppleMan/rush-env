#[derive(Debug, Clone)]
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

impl BorderStyle {
    pub fn simple() -> Self {
        BorderStyle {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '─',
            vertical: '│',
            size: 1,
        }
    }

    pub fn single() -> Self {
        BorderStyle {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
            size: 1,
        }
    }

    pub fn double() -> Self {
        BorderStyle {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
            size: 1,
        }
    }

    pub fn rounded() -> Self {
        BorderStyle {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
            size: 1,
        }
    }

    pub fn heavy() -> Self {
        BorderStyle {
            top_left: '┏',
            top_right: '┓',
            bottom_left: '┗',
            bottom_right: '┛',
            horizontal: '━',
            vertical: '┃',
            size: 1,
        }
    }

    pub fn block() -> Self {
        BorderStyle {
            top_left: '█',
            top_right: '█',
            bottom_left: '█',
            bottom_right: '█',
            horizontal: '█',
            vertical: '█',
            size: 1,
        }
    }

    pub fn dotted() -> Self {
        BorderStyle {
            top_left: '.',
            top_right: '.',
            bottom_left: ':',
            bottom_right: ':',
            horizontal: '.',
            vertical: ':',
            size: 1,
        }
    }

    pub fn ascii_light() -> Self {
        BorderStyle {
            top_left: '/',
            top_right: '\\',
            bottom_left: '\\',
            bottom_right: '/',
            horizontal: '-',
            vertical: '|',
            size: 1,
        }
    }

    pub fn angled() -> Self {
        BorderStyle {
            top_left: '╱',
            top_right: '╲',
            bottom_left: '╲',
            bottom_right: '╱',
            horizontal: '─',
            vertical: '│',
            size: 1,
        }
    }

    pub fn thin_double() -> Self {
        BorderStyle {
            top_left: '╒',
            top_right: '╕',
            bottom_left: '╘',
            bottom_right: '╛',
            horizontal: '─',
            vertical: '│',
            size: 1,
        }
    }

    pub fn square() -> Self {
        BorderStyle {
            top_left: '■',
            top_right: '■',
            bottom_left: '■',
            bottom_right: '■',
            horizontal: '■',
            vertical: '■',
            size: 1,
        }
    }

    pub fn dashed_round() -> Self {
        BorderStyle {
            top_left: '●',
            top_right: '●',
            bottom_left: '●',
            bottom_right: '●',
            horizontal: '•',
            vertical: '•',
            size: 1,
        }
    }

    pub fn wave() -> Self {
        BorderStyle {
            top_left: '≈',
            top_right: '≈',
            bottom_left: '≈',
            bottom_right: '≈',
            horizontal: '≈',
            vertical: '∣',
            size: 1,
        }
    }

    pub fn zigzag() -> Self {
        BorderStyle {
            top_left: '╱',
            top_right: '╲',
            bottom_left: '╲',
            bottom_right: '╱',
            horizontal: '╌',
            vertical: '╎',
            size: 1,
        }
    }

    pub fn dotted_box() -> Self {
        BorderStyle {
            top_left: '∙',
            top_right: '∙',
            bottom_left: '∙',
            bottom_right: '∙',
            horizontal: '·',
            vertical: '·',
            size: 1,
        }
    }

    pub fn triangle_chain() -> Self {
        BorderStyle {
            top_left: '◤',
            top_right: '◥',
            bottom_left: '◣',
            bottom_right: '◢',
            horizontal: '▲',
            vertical: '▶',
            size: 1,
        }
    }

    pub fn ascii_flower() -> Self {
        BorderStyle {
            top_left: '*',
            top_right: '*',
            bottom_left: '*',
            bottom_right: '*',
            horizontal: '~',
            vertical: '|',
            size: 1,
        }
    }
}
