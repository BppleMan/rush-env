use crate::core::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};

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
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            condition: Condition::default(),
        }
    }

    pub fn tag() -> &'static str {
        "<export name>"
    }

    pub fn export(name: impl AsRef<str>, value: impl AsRef<str>, buf: &mut impl std::io::Write) -> Result<(), VisitorError> {
        let name = name.as_ref();
        let value = value.as_ref();
        if name.to_uppercase() == "PATH" {
            return Err(VisitorError::ExportPath(value.to_string()));
        }
        unsafe {
            std::env::set_var(name, value);
        }
        writeln!(buf, r#"export {name}="{value}""#)?;
        Ok(())
    }
}

impl Visit for ExportScript {
    fn visit<'a>(&'a self, _context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        Self::export(&self.name, &self.value, writer)
    }
}
