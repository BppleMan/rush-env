use crate::style::BorderStyle;
use crate::widget::{Align, Constraints, Size, Widget};
use std::io::Write;

#[derive(Default, Debug, Clone)]
pub struct Container<T>
where
    T: Widget,
{
    pub inner: T,
    pub border: BorderStyle,
    pub padding: usize,
    pub margin: usize,
    pub align: Align,
    size: Size,
}

impl<T> Container<T>
where
    T: Widget,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            border: BorderStyle::default(),
            padding: 0,
            margin: 0,
            align: Align::Center,
            size: Size::default(),
        }
    }

    pub fn set_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn set_padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub fn set_margin(mut self, margin: usize) -> Self {
        self.margin = margin;
        self
    }

    pub fn set_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

impl<T> Widget for Container<T>
where
    T: Widget,
{
    fn size(&self) -> Size {
        self.size
    }

    fn layout(&mut self, constraints: Constraints) {
        println!("margin: {}, padding: {}", self.margin, self.padding);
        let left = self.margin + self.padding + self.border.size;
        let right = self.margin + self.padding + self.border.size;
        let top = self.border.size;
        let bottom = self.border.size;
        let constraints = Constraints {
            max_width: constraints.max_width.saturating_sub(left + right),
        };
        self.inner.layout(constraints);
        println!("Container layout: {:?}", self.inner.size());
        self.size = Size {
            width: self.inner.size().width + left + right,
            height: self.inner.size().height + top + bottom,
        };
    }

    fn render(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        write!(writer, "{}", " ".repeat(self.margin))?;
        if row < self.border.size {
            let repeat = self.size.width.saturating_sub(2 * self.border.size + 2 * self.margin);
            write!(
                writer,
                "{}{}{}",
                self.border.top_left,
                self.border.horizontal.to_string().repeat(repeat),
                self.border.top_right
            )?;
        } else if row >= self.size.height - self.border.size {
            let repeat = self.size.width.saturating_sub(2 * self.border.size + 2 * self.margin);
            write!(
                writer,
                "{}{}{}",
                self.border.bottom_left,
                self.border.horizontal.to_string().repeat(repeat),
                self.border.bottom_right
            )?;
        } else {
            let pad = self
                .size
                .width
                .saturating_sub(2 * self.margin + 2 * self.border.size + 2 * self.padding + self.inner.size().width);
            println!("self.size: {:?}, inner.size: {:?}, pad: {pad}", self.size, self.inner.size());
            let (left, right) = match self.align {
                Align::Center => (pad / 2, pad - (pad / 2)),
                Align::Left => (0, pad),
                Align::Right => (pad, 0),
            };
            write!(writer, "{}{}", self.border.vertical, " ".repeat(self.padding + left))?;
            self.inner.render(writer, row - self.border.size)?;
            write!(writer, "{}{}", " ".repeat(self.padding + right), self.border.vertical)?;
        }
        write!(writer, "{}", " ".repeat(self.margin))
    }
}
