use crate::core::condition::Condition;
use crate::core::path::Paths;
use crate::core::script::Scripts;
use crate::core::script::export::ExportScript;
use crate::visitor::{Visitor, VisitorContext, VisitorError};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Language {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@version", default)]
    pub version: Option<String>,
    pub description: String,
    #[serde(default)]
    pub condition: Condition,
    #[serde(default)]
    pub scripts: Scripts,
    #[serde(default)]
    pub paths: Paths,
}

#[derive(Default, Debug, Clone, Serialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
pub struct Languages(pub Vec<Language>);

impl Language {
    pub fn tag() -> &'static str {
        "<language>"
    }
}

impl Visitor for Language {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        if let Some(version) = &self.version {
            let name = format!("{}_VERSION", self.name.to_uppercase());
            let value = version.clone();
            ExportScript::export(name, value, &mut context.script)?;
        }
        self.paths.visit(context)?;
        self.scripts.visit(context)?;
        Ok(())
    }
}

impl Visitor for Languages {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        for language in &self.0 {
            language.visit(context)?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Languages {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct List {
            #[serde(rename = "$value", default)]
            element: Vec<Language>,
        }
        Ok(Languages(List::deserialize(deserializer)?.element))
    }
}
