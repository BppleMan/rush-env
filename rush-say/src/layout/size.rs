use crate::layout::pos::Pos;
use rush_macros::Getter;
use std::fmt::{self, Display};
use std::ops::{Add, AddAssign};

#[derive(Default, Debug, Clone, Copy, Getter)]
pub struct Size {
    w: usize,
    h: usize,
}

impl Size {
    pub fn new(w: usize, h: usize) -> Self {
        Size { w, h }
    }

    pub fn w(&self) -> usize {
        self.w
    }

    pub fn h(&self) -> usize {
        self.h
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.w, self.h)
    }
}

impl Size {
    pub fn top_left_of(&self, _size: &Size) -> Pos {
        Pos::new(0, 0)
    }

    pub fn top_center_of(&self, size: &Size) -> Pos {
        Pos::new(self.w.saturating_sub(size.w) / 2, 0)
    }

    pub fn top_right_of(&self, size: &Size) -> Pos {
        Pos::new(self.w.saturating_sub(size.w) / 2, 0)
    }

    pub fn left_of(&self, size: &Size) -> Pos {
        Pos::new(0, self.h.saturating_sub(size.h) / 2)
    }

    pub fn center_of(&self, size: &Size) -> Pos {
        Pos::new(self.w.saturating_sub(size.w) / 2, self.h.saturating_sub(size.h) / 2)
    }

    pub fn right_of(&self, size: &Size) -> Pos {
        Pos::new(self.w.saturating_sub(size.w), self.h.saturating_sub(size.h) / 2)
    }

    pub fn bottom_left_of(&self, size: &Size) -> Pos {
        Pos::new(0, self.h.saturating_sub(size.h))
    }

    pub fn bottom_center_of(&self, size: &Size) -> Pos {
        Pos::new(self.w.saturating_sub(size.w) / 2, self.h.saturating_sub(size.h))
    }

    pub fn bottom_right_of(&self, size: &Size) -> Pos {
        Pos::new(self.w.saturating_sub(size.w), self.h.saturating_sub(size.h))
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w + rhs.w,
            h: self.h + rhs.h,
        }
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        self.w += rhs.w;
        self.h += rhs.h;
    }
}

#[cfg(test)]
mod tests {
    use super::Size;
    use color_eyre::Result;

    #[test]
    fn test_generated_getter_keeps_size_fields_observable() -> Result<()> {
        let _ = color_eyre::install();

        let size = Size::new(3, 4);

        assert_eq!(*size.get_w(), 3);
        assert_eq!(*size.get_h(), 4);
        assert_eq!(size.w(), 3);
        assert_eq!(size.h(), 4);

        Ok(())
    }
}
