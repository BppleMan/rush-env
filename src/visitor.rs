use crate::core::path::Path;
use thiserror::Error;

#[derive(Default, Debug)]
pub struct VisitorContext<'a> {
    pub paths: Vec<&'a Path>,
    pub script: String,
}

pub trait Visitor {
    fn visit<'a>(&'a self, _context: &mut VisitorContext<'a>) -> Result<(), VisitorError>;
}

// pub trait Render {
//     fn render_script<W: Write>(&self, _output: &mut W) -> Result<(), VisitorError>;
// }
//
// pub trait CollectPath {
//     fn collect_path<'a, 'b>(&'a self, _paths: &'b mut Vec<&'a Path>) -> Result<(), VisitorError>
//     where
//         'a: 'b;
// }

#[derive(Debug, Error)]
pub enum VisitorError {
    #[error("Cannot collect path on script: {0}.")]
    CollectPathOnScript(&'static str),

    #[error("Cannot export ${{PATH}} using '<export>': {0}. Consider using '<path>' instead.")]
    ExportPath(String),

    #[error("Not found source file: {0}.")]
    SourceFileNotExist(String),

    #[error(transparent)]
    JoinPathsError(#[from] std::env::JoinPathsError),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}
