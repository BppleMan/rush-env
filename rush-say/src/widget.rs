mod text;
mod container;

pub use container::Container;
pub use text::Text;

pub trait Widget {
    fn size(&self) -> Size;

    fn layout(&mut self, constraints: Constraints);

    fn render(&self, writer: &mut impl std::io::Write, row: usize) -> std::io::Result<()>;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Constraints {
    pub render_width: usize,
    pub wrap_width: usize,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Default, Debug, Clone)]
pub enum Align {
    #[default]
    Center,
    Left,
    Right,
}

pub fn visual_width_chars<'a>(chars: &[char]) -> usize {
    chars.iter().copied().map(visual_width_char).sum()
}

/// 视觉宽度换算（可随时自定义规则）
pub fn visual_width_char(ch: char) -> usize {
    match ch {
        '\u{4e00}'..='\u{9fff}'   // CJK汉字
        | '\u{3000}'..='\u{303f}' // CJK标点
        | '\u{3040}'..='\u{30ff}' // 日文
        | '\u{ff00}'..='\u{ffef}' // 全角
        => 2,
        '\u{1f300}'..='\u{1f6ff}' | '\u{1f900}'..='\u{1f9ff}' => 2, // emoji
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::style::BorderStyle;
    use crate::widget::container::Container;
    use crate::widget::text::Text;
    use crate::widget::{Align, Constraints, Widget};
    use std::io::{Cursor, Write};

    #[test]
    fn test_render_text() {
        let mut text = Text::new("11223344556677889900").set_align(Align::Center);
        text.layout(Constraints {
            render_width: 10,
            wrap_width: 4,
        });
        let mut output = Cursor::new(vec![]);
        for row in 0..text.size().height {
            text.render(&mut output, row).unwrap();
            writeln!(&mut output).unwrap();
        }
        println!("{}", String::from_utf8(output.into_inner()).unwrap());
    }

    #[test]
    fn test_render_container() {
        let text = Text::new("001122334455667788990011223344556677889900112233445566778899").set_align(Align::Center);
        let mut container = Container::new(text)
            .set_align(Align::Center)
            .set_margin(1)
            .set_padding(10)
            .set_border(BorderStyle::dashed_round());
        container.layout(Constraints {
            render_width: 40,
            wrap_width: 40,
        });
        let mut output = Cursor::new(vec![]);
        for row in 0..container.size().height {
            container.render(&mut output, row).unwrap();
        }
        println!("{}", String::from_utf8(output.into_inner()).unwrap());
    }
}
