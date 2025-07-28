use crate::core::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RawScript {
    #[serde(rename = "$text")]
    pub script: String,
    #[serde(default)]
    pub condition: Condition,
}

impl RawScript {
    pub fn tag() -> &'static str {
        "<raw>"
    }
}

impl Visit for RawScript {
    fn visit<'a>(&'a self, _context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        writeln!(writer, "{}", self.script)?;
        Ok(())
    }
}
