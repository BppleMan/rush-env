use crate::core::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};

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

impl Visit for EvalScript {
    fn visit<'a>(&'a self, _context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        writeln!(writer, r#"eval $({})"#, self.script)?;
        Ok(())
    }
}
