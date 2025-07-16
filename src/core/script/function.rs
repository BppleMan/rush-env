use crate::core::condition::Condition;
use crate::visitor::{Render, Visitor, VisitorContext, VisitorError};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FunctionScript {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub body: String,
    #[serde(default)]
    pub condition: Condition,
}

impl FunctionScript {
    pub fn tag() -> &'static str {
        "<function name>"
    }
}

impl Visitor for FunctionScript {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        self.render_script(&mut context.script)
    }
}

impl Render for FunctionScript {
    fn render_script<W: Write>(&self, output: &mut W) -> Result<(), VisitorError> {
        writeln!(output, "function {} {{", self.name)?;
        let lines = self
            .body
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| format!("    {l}"))
            .collect::<Vec<String>>()
            .join("\n");
        writeln!(output, "{lines}")?;
        writeln!(output, "}}")?;
        Ok(())
    }
}
