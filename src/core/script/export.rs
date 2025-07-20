use crate::core::condition::Condition;
use crate::visitor::{Visitor, VisitorContext, VisitorError};
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

    pub fn export(name: impl AsRef<str>, value: impl AsRef<str>, script: &mut String) -> Result<(), VisitorError> {
        let name = name.as_ref();
        let value = value.as_ref();
        if name.to_uppercase() == "PATH" {
            return Err(VisitorError::ExportPath(value.to_string()));
        }
        unsafe {
            std::env::set_var(name, value);
        }
        writeln!(script, r#"export {name} = "{value}""#)?;
        Ok(())
    }
}

impl Visitor for ExportScript {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        Self::export(&self.name, &self.value, &mut context.script)
    }
}
