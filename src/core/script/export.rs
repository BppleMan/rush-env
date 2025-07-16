use crate::core::condition::Condition;
use crate::visitor::{Render, Visitor, VisitorContext, VisitorError};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExportScript {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub value: String,
    #[serde(default)]
    pub condition: Condition,
}

impl ExportScript {
    pub fn tag() -> &'static str {
        "<export name>"
    }
}

impl Visitor for ExportScript {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        if self.name.to_uppercase() == "PATH" {
            return Err(VisitorError::ExportPath(self.value.clone()));
        }
        self.render_script(&mut context.script)
    }
}

impl Render for ExportScript {
    fn render_script<W: Write>(&self, output: &mut W) -> Result<(), VisitorError> {
        Ok(writeln!(output, r#"export {} = "{}""#, self.name, self.value)?)
    }
}
