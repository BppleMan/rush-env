use crate::core::installer::Install;
use crate::core::rush::condition::Condition;
use crate::core::rush::script::Script;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Plugin {
    pub name: String,
    pub work_dir: String,
    #[serde(default)]
    pub condition: Condition,
    pub scripts: Vec<Script>,
    #[serde(default)]
    pub install: Install,
}

impl Plugin {
    pub fn tag() -> &'static str {
        "<plugin name work_dir>"
    }
}

impl Visit for Plugin {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        // if !self.condition.check() {
        //     return Ok(());
        // }
        // let name = format!("{}_DIR", self.name.to_uppercase());
        // let value = self.work_dir.clone();
        // ExportScript::export(name, value, writer)?;
        // self.scripts.visit(context, writer)?;
        Ok(())
    }
}
