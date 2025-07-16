use crate::core::condition::Condition;
use crate::core::path::{Path, Paths};
use crate::core::script::Scripts;
use crate::visitor::{CollectPath, Render, Visitor, VisitorContext, VisitorError};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Write;

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
        self.collect_path(&mut context.paths)?;
        self.render_script(&mut context.script)?;
        self.scripts.visit(context)?;
        Ok(())
    }
}

impl Render for Language {
    fn render_script<W: Write>(&self, output: &mut W) -> Result<(), VisitorError> {
        if let Some(version) = &self.version {
            writeln!(output, r#"export {}_VERSION = "{}""#, self.name.to_uppercase(), version)?;
        }
        Ok(())
    }
}

impl CollectPath for Language {
    fn collect_path<'a, 'b>(&'a self, paths: &'b mut Vec<&'a Path>) -> Result<(), VisitorError>
    where
        'a: 'b,
    {
        self.paths.collect_path(paths)
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
