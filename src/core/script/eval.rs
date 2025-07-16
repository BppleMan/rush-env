use crate::core::condition::Condition;
use crate::visitor::{Render, Visitor, VisitorContext, VisitorError};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EvalScript {
    #[serde(rename = "$text")]
    pub script: String,
    #[serde(default)]
    pub condition: Condition,
}

impl EvalScript {
    pub fn tag() -> &'static str {
        "<eval>"
    }
}

impl Visitor for EvalScript {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        self.render_script(&mut context.script)
    }
}

impl Render for EvalScript {
    fn render_script<W: Write>(&self, output: &mut W) -> Result<(), VisitorError> {
        Ok(writeln!(output, r#"eval $({})"#, self.script)?)
    }
}
