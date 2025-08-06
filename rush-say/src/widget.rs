pub use constraints::Constraints;
pub use size::Size;

mod constraints;
mod size;

pub trait Widget: Layout + Renderable {}

pub trait Layout {
    fn layout(&self, constraints: Constraints) -> Size;
}

pub trait Renderable {
    fn render(&self, writer: &mut impl std::io::Write) -> std::io::Result<()>;
}
