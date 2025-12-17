use crate::core::rush::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AliasScript {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub condition: Condition,
}

impl AliasScript {
    pub fn tag() -> &'static str {
        "<alias name>"
    }
}

impl Visit for AliasScript {
    fn visit<'a>(&'a self, _context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        writeln!(writer, r#"alias {} = "{}""#, self.name, self.command)?;
        Ok(())
    }
}
