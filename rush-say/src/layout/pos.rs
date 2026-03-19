use rush_macros::Getter;
use std::fmt::{self, Display};
use std::ops::{Add, AddAssign};

#[derive(Default, Debug, Clone, Copy, Getter)]
pub struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    pub fn new(x: usize, y: usize) -> Self {
        Pos { x, y }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[cfg(test)]
mod tests {
    use super::Pos;
    use color_eyre::Result;

    #[test]
    fn test_generated_getter_keeps_pos_fields_observable() -> Result<()> {
        let _ = color_eyre::install();

        let pos = Pos::new(5, 6);

        assert_eq!(*pos.get_x(), 5);
        assert_eq!(*pos.get_y(), 6);
        assert_eq!(pos.x(), 5);
        assert_eq!(pos.y(), 6);

        Ok(())
    }
}
