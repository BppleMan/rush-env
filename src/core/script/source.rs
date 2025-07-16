use crate::core::condition::Condition;
use crate::visitor::{Render, Visitor, VisitorContext, VisitorError};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

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
        // let file = std::path::Path::new(&self.file);
        // if !file.is_file() {
        //     return Err(VisitorError::SourceFileNotExist(self.file.clone()));
        // }
        self.render_script(&mut context.script)
    }
}

impl Render for SourceScript {
    fn render_script<W: Write>(&self, output: &mut W) -> Result<(), VisitorError> {
        Ok(writeln!(output, "source {}", self.file)?)
    }
}
