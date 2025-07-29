use crate::core::condition::Condition;
use crate::core::path::Paths;
use crate::core::script::Scripts;
use crate::core::script::export::ExportScript;
use crate::visitor::{Visit, Visitor, VisitorError};
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

impl Visit for Language {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        if let Some(version) = &self.version {
            let name = format!("{}_VERSION", self.name.to_uppercase());
            let value = version.clone();
            ExportScript::export(name, value, writer)?;
        }
        self.paths.visit(context, writer)?;
        self.scripts.visit(context, writer)?;
        Ok(())
    }
}

impl Visit for Languages {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        for language in &self.0 {
            language.visit(context, writer)?;
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
