use crate::layout::{Align, Rect, RectTrait, Size};
use crate::widget::Widget;
use std::fmt::{self, Display};
use std::io::Write;

/// 渲染逻辑盒模型
#[derive(Default, Debug, Copy, Clone)]
pub struct LogicBox {
    /// 逻辑盒自身 Rect
    pub rect: Rect,
    /// 逻辑盒包装物 Rect
    pub child: Rect,

    pub align: Align,
}

impl Display for LogicBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rect={} | child={} | align={:?}", self.rect, self.child, self.align)
    }
}

impl LogicBox {
    pub fn new(rect: Rect, align: Align) -> Self {
        LogicBox {
            rect,
            align,
            ..Default::default()
        }
    }

    pub fn set_child_size(&mut self, size: Size) {
        self.child.pos = self.align.calc_relative_pos(&self.rect.size, &size);
        self.child.size = size;
    }
}

#[allow(unused)]
impl LogicBox {
    pub fn offset(&mut self, dx: usize, dy: usize) {
        self.rect.offset(dx, dy)
    }

    pub fn layout(&mut self, child_size: Size) {
        self.child = self.layout_child(child_size);
    }

    fn layout_child(&self, child_size: Size) -> Rect {
        let size = &self.rect.size;
        let pos = self.align.calc_relative_pos(size, &child_size);
        Rect::pos_size(pos, child_size)
    }

    pub fn render<T>(&self, writer: &mut impl Write, child: &T, row: usize) -> std::io::Result<()>
    where
        T: Widget,
    {
        match row {
            // 渲染高于 child 的上方空行, 长度为 rect 宽度
            r if r < self.child.top() => write!(writer, "{}", " ".repeat(self.rect.w()))?,
            // 渲染低于 child 的下方空行, 长度为 rect 宽度
            r if r > self.child.bottom() => write!(writer, "{}", " ".repeat(self.rect.w()))?,
            // row 落在 child 范围内时, 渲染 child
            _ => self.render_child(writer, child, self.child.relative_row(row))?,
        };
        Ok(())
    }

    fn render_child<T>(&self, writer: &mut impl Write, child: &T, row: usize) -> std::io::Result<()>
    where
        T: Widget,
    {
        let left_space = self.child.left();
        let right_space = self.rect.w().saturating_sub(self.child.right() + 1);
        // 渲染 child 左边空白区域
        write!(writer, "{}", " ".repeat(left_space))?;
        // 渲染 child 本体
        child.render(writer, row)?;
        // 渲染 child 右边空白区域
        write!(writer, "{}", " ".repeat(right_space))?;
        Ok(())
    }
}
