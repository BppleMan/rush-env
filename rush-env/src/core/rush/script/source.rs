use crate::core::rush::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use rush_var::expand_var_env;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SourceScript {
    #[serde(rename = "$text")]
    pub file: String,
    #[serde(default)]
    pub condition: Condition,
}

impl SourceScript {
    pub fn tag() -> &'static str {
        "<source>"
    }
}

impl Visit for SourceScript {
    fn visit<'a>(&'a self, _context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        let expanded_file_path = expand_var_env(&self.file);
        let file = PathBuf::from(expanded_file_path);
        if !file.is_file() {
            return Err(VisitorError::SourceFileNotExist(self.file.clone()));
        }
        writeln!(writer, "source {}", self.file)?;
        Ok(())
    }
}
