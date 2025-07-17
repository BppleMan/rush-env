use crate::core::condition::Condition;
use crate::visitor::{Visitor, VisitorContext, VisitorError};
use rush_var::expand_env_vars;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
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

impl Visitor for SourceScript {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        let expanded_file_path = expand_env_vars(&self.file);
        let file = PathBuf::from(expanded_file_path);
        if !file.is_file() {
            return Err(VisitorError::SourceFileNotExist(self.file.clone()));
        }
        writeln!(context.script, "source {}", self.file)?;
        Ok(())
    }
}
