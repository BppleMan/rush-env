use crate::core::rush::condition::Condition;
use crate::core::rush::path::Path;
use crate::core::rush::script::Script;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Language {
    pub name: String,
    pub enable: bool,
    pub version: Option<String>,
    pub desc: Option<String>,
    #[serde(default)]
    pub condition: Condition,
    #[serde(default)]
    pub scripts: Vec<Script>,
    #[serde(default)]
    pub paths: Vec<Path>,
}

impl Language {
    pub fn tag() -> &'static str {
        "<language>"
    }
}

impl Visit for Language {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        // if !self.condition.check() {
        //     return Ok(());
        // }
        // if let Some(version) = &self.version {
        //     let name = format!("{}_VERSION", self.name.to_uppercase());
        //     let value = version.clone();
        //     ExportScript::export(name, value, writer)?;
        // }
        // self.paths.visit(context, writer)?;
        // self.scripts.visit(context, writer)?;
        Ok(())
    }
}
