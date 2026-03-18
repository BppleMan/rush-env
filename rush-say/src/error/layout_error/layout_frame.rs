use rush_ext::Getter;
use rush_ext::Setter;

use super::WidgetView;
use crate::layout::Constraints;
use crate::layout::Size;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Setter, Getter)]
pub struct LayoutFrame {
    view: WidgetView,
    size: Size,
    constraints: Constraints,
    // This is not known when the child frame is created.
    // A parent attaches it only when bubbling the child's LayoutError upward.
    sibling_index: Option<SiblingIndex>,
}

impl LayoutFrame {
    pub fn new(view: WidgetView, size: Size, constraints: Constraints) -> Self {
        Self {
            view,
            size,
            constraints,
            sibling_index: None,
        }
    }

    pub(crate) fn with_sibling_index(mut self, sibling_index: SiblingIndex) -> Self {
        self.sibling_index = Some(sibling_index);
        self
    }

    pub(crate) fn sibling_index(&self) -> Option<SiblingIndex> {
        self.sibling_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiblingIndex {
    Only,
    Index { index: usize, total: usize },
}

impl SiblingIndex {
    pub const fn index(index: usize, total: usize) -> Self {
        Self::Index { index, total }
    }

    pub const fn label(self) -> Option<usize> {
        match self {
            Self::Only => None,
            Self::Index { index, .. } => Some(index),
        }
    }

    pub const fn shows_before_ellipsis(self) -> bool {
        match self {
            Self::Only => false,
            Self::Index { index, total } => total > 1 && index > 0,
        }
    }

    pub const fn shows_after_ellipsis(self) -> bool {
        match self {
            Self::Only => false,
            Self::Index { index, total } => total > 1 && index + 1 < total,
        }
    }
}

impl Display for LayoutFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sibling_index) = self.sibling_index
            && let Some(index) = sibling_index.label()
        {
            write!(f, "[{index}] ")?;
        }
        write!(f, "{} {{ {} x {} }} ", self.view, self.size.w(), self.size.h(),)?;
        if self.constraints.max_height == 0 {
            write!(f, "|-> {} <-|", self.constraints.max_width)?;
        } else {
            write!(f, "|-> {} x {} <-|", self.constraints.max_width, self.constraints.max_height)?;
        }
        self.view.write_attrs(f)
    }
}
