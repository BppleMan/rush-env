use crate::layout::pos::Pos;
use crate::layout::size::Size;
use std::fmt::{self, Display};

pub trait RectTrait {
    fn pos(&self) -> Pos;

    fn size(&self) -> Size;

    fn x(&self) -> usize {
        self.pos().x()
    }

    fn y(&self) -> usize {
        self.pos().y()
    }

    fn w(&self) -> usize {
        self.size().w()
    }

    fn h(&self) -> usize {
        self.size().h()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Rect {
    pub pos: Pos,
    pub size: Size,
}

impl Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.pos, self.size)
    }
}

/// 创建
impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Rect {
            pos: Pos::new(x, y),
            size: Size::new(w, h),
        }
    }

    pub fn size_with_x_y(size: Size, x: usize, y: usize) -> Self {
        Rect { pos: Pos::new(x, y), size }
    }

    pub fn pos_with_w_h(pos: Pos, w: usize, h: usize) -> Self {
        Rect {
            pos,
            size: Size::new(w, h),
        }
    }

    pub fn pos_size(pos: Pos, size: Size) -> Self {
        Rect { pos, size }
    }
}

impl RectTrait for Rect {
    fn pos(&self) -> Pos {
        self.pos
    }

    fn size(&self) -> Size {
        self.size
    }
}

impl Rect {
    pub fn top(&self) -> usize {
        self.pos.y()
    }

    pub fn bottom(&self) -> usize {
        if self.size.h() < 1 {
            panic!("[Rect] 无法计算 bottom, 预期 h >= 1, 实际 h = {}", self.size.h());
        }
        self.pos.y() + self.size.h() - 1
    }

    pub fn left(&self) -> usize {
        self.pos.x()
    }

    pub fn right(&self) -> usize {
        if self.size().w() < 1 {
            panic!("[Rect] 无法计算 right, 预期 w >= 1, 实际 w = {}", self.size.w());
        }
        self.pos.x() + self.size.w() - 1
    }

    pub fn top_left(&self) -> Pos {
        Pos::new(self.left(), self.top())
    }

    pub fn top_right(&self) -> Pos {
        Pos::new(self.right(), self.top())
    }

    pub fn bottom_left(&self) -> Pos {
        Pos::new(self.left(), self.bottom())
    }

    pub fn bottom_right(&self) -> Pos {
        Pos::new(self.right(), self.bottom())
    }

    pub fn inside(&self, pos: &Pos) -> bool {
        pos.x() >= self.left() && pos.x() <= self.right() && pos.y() >= self.top() && pos.y() <= self.bottom()
    }

    pub fn relative_row(&self, row: usize) -> usize {
        if row < self.top() {
            panic!(
                "[Rect] 无法计算 relative_row, 预期 row >= top, 实际 row = {}, top = {}",
                row,
                self.top()
            );
        }
        row - self.top()
    }
}

impl Rect {
    pub fn offset(&mut self, dx: usize, dy: usize) {
        self.pos += Pos::new(dx, dy);
    }
}
