use crate::core::condition::Condition;
use crate::visitor::{Render, Visitor, VisitorContext, VisitorError};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AliasScript {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub command: String,
    #[serde(default)]
    pub condition: Condition,
}

impl AliasScript {
    pub fn tag() -> &'static str {
        "<alias name>"
    }
}

impl Visitor for AliasScript {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        self.render_script(&mut context.script)
    }
}

impl Render for AliasScript {
    fn render_script<W: Write>(&self, output: &mut W) -> Result<(), VisitorError> {
        Ok(writeln!(output, r#"alias {} = "{}""#, self.name, self.command)?)
    }
}
