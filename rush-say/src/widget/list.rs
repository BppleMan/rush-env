use crate::{Align, BorderStyle, Widget};

#[derive(Default, Debug, Clone)]
pub struct List<T>
where
    T: Widget,
{
    pub inner: Vec<T>,
    pub border: BorderStyle,
    pub padding: usize,
    pub margin: usize,
    pub align: Align,
}
