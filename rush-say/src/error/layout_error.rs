mod layout_diagnostic;
mod layout_frame;
mod widget_view;

pub use layout_diagnostic::*;
pub use layout_frame::*;
pub use widget_view::*;

use std::error::Error;
use std::fmt::{self, Display};

pub type LayoutResult<T = ()> = Result<T, LayoutError>;

#[derive(Debug, Clone)]
pub struct LayoutError {
    path: Vec<LayoutFrame>,
    diagnostic: LayoutDiagnostic,
}

impl LayoutError {
    pub fn leaf(frame: LayoutFrame, diagnostic: LayoutDiagnostic) -> Self {
        Self {
            path: vec![frame],
            diagnostic,
        }
    }

    pub fn in_parent(mut self, frame: LayoutFrame, sibling_index: SiblingIndex) -> Self {
        if let Some(child_root) = self.path.first_mut() {
            *child_root = child_root.clone().with_sibling_index(sibling_index);
        }
        self.path.insert(0, frame);
        self
    }
}

pub trait LayoutResultExt<T> {
    fn with_parent(self, frame: LayoutFrame, branch: SiblingIndex) -> LayoutResult<T>;
}

impl<T> LayoutResultExt<T> for LayoutResult<T> {
    fn with_parent(self, frame: LayoutFrame, branch: SiblingIndex) -> LayoutResult<T> {
        self.map_err(|error| error.in_parent(frame, branch))
    }
}

impl Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LayoutError: {}", self.diagnostic)?;
        writeln!(f)?;
        if !self.path.is_empty() {
            writeln!(f, "{}", PathDisplay { path: &self.path })?;
        }
        writeln!(f)?;
        self.diagnostic.write_evidence(f)
    }
}

impl Error for LayoutError {}

struct PathDisplay<'a> {
    path: &'a [LayoutFrame],
}

impl Display for PathDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((root, descendants)) = self.path.split_first() else {
            return Ok(());
        };

        write!(f, "{root}")?;
        for (depth, child) in descendants.iter().enumerate() {
            write!(f, "\n{}", BranchDisplay { depth, frame: child })?;
        }
        Ok(())
    }
}

struct BranchDisplay<'a> {
    depth: usize,
    frame: &'a LayoutFrame,
}

impl Display for BranchDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = "    ".repeat(self.depth);
        let branch = self
            .frame
            .sibling_index()
            .expect("non-root frame must describe how it appears under its parent");

        write!(f, "{indent}  |")?;
        if branch.shows_before_ellipsis() {
            write!(f, "\n{indent}  +-- ...")?;
            write!(f, "\n{indent}  |")?;
        }

        write!(f, "\n{indent}  +-- {}", self.frame)?;

        if branch.shows_after_ellipsis() {
            write!(f, "\n{indent}  |")?;
            write!(f, "\n{indent}  +-- ...")?;
        }
        Ok(())
    }
}
