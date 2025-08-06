#[derive(Default, Debug, Clone, Copy)]
pub struct Constraints {
    pub min_width: usize,
    pub max_width: usize,
    pub min_height: usize,
    pub max_height: usize,
}
