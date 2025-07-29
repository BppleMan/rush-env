use crate::core::condition::Condition;
use crate::core::script::Scripts;
use crate::core::script::export::ExportScript;
use crate::visitor::{Visit, Visitor, VisitorError};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Plugin {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@work_dir")]
    pub work_dir: String,
    #[serde(default)]
    pub condition: Condition,
    pub scripts: Scripts,
}

#[derive(Default, Debug, Clone, Serialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
pub struct Plugins(pub Vec<Plugin>);

impl Plugin {
    pub fn tag() -> &'static str {
        "<plugin name work_dir>"
    }
}

impl Visit for Plugin {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        let name = format!("{}_DIR", self.name.to_uppercase());
        let value = self.work_dir.clone();
        ExportScript::export(name, value, writer)?;
        self.scripts.visit(context, writer)?;
        Ok(())
    }
}

impl Visit for Plugins {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        for plugin in self.0.iter() {
            context.plugin_work_dirs.push(&plugin.work_dir);
            plugin.visit(context, writer)?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Plugins {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct List {
            #[serde(rename = "$value", default)]
            element: Vec<Plugin>,
        }
        Ok(Plugins(List::deserialize(deserializer)?.element))
    }
}
