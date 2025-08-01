use crate::core::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VarScript {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub value: String,
    #[serde(default)]
    pub condition: Condition,
}

#[derive(Parser)]
struct Cli {
    #[clap(default_value = "Random", value_enum)]
    pub password_type: PasswordType,
    #[clap(default_value = "16")]
    pub length: usize,
    #[clap(long = "a", default_value = "true")]
    pub alpha: bool,
    #[clap(long = "b", default_value = "true")]
    pub numeric: bool,
}

#[derive(Default, Clone, ValueEnum)]
enum PasswordType {
    #[default]
    Random,
    Easy,
    Pin,
}

impl VarScript {
    pub fn tag() -> &'static str {
        "<var name>"
    }
}

impl Visit for VarScript {
    fn visit<'a>(&'a self, _context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        writeln!(writer, r#"{}="{}""#, self.name, self.value)?;
        Ok(())
    }
}
