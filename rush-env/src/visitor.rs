use crate::core::path::Path;
use rush_say::Section;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Default, Debug)]
pub struct Visitor<'a> {
    pub rush_dir: PathBuf,
    pub section: Section,
    pub paths: Vec<&'a Path>,
    pub plugin_work_dirs: Vec<&'a str>,
}

pub trait Visit {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError>;
}

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

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
