use crate::core::script::alias::AliasScript;
use crate::core::script::eval::EvalScript;
use crate::core::script::export::ExportScript;
use crate::core::script::function::FunctionScript;
use crate::core::script::raw::RawScript;
use crate::core::script::source::SourceScript;
use crate::core::script::var::VarScript;
use crate::visitor::{Render, Visitor, VisitorContext, VisitorError};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Write;

pub mod alias;
pub mod eval;
pub mod export;
pub mod function;
pub mod raw;
pub mod source;
pub mod var;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Script {
    #[serde(rename = "alias")]
    Alias(AliasScript),
    #[serde(rename = "eval")]
    Eval(EvalScript),
    #[serde(rename = "export")]
    Export(ExportScript),
    #[serde(rename = "function")]
    Function(FunctionScript),
    #[serde(rename = "raw")]
    Raw(RawScript),
    #[serde(rename = "source")]
    Source(SourceScript),
    #[serde(rename = "var")]
    Var(VarScript),
    #[default]
    None,
}

#[derive(Default, Debug, Clone, Serialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
pub struct Scripts(pub Vec<Script>);

impl<'de> Deserialize<'de> for Scripts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct List {
            #[serde(rename = "$value", default)]
            element: Vec<Script>,
        }
        Ok(Scripts(List::deserialize(deserializer)?.element))
    }
}

impl Visitor for Script {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        match self {
            Script::Alias(alias) => alias.visit(context),
            Script::Eval(eval) => eval.visit(context),
            Script::Export(export) => export.visit(context),
            Script::Function(function) => function.visit(context),
            Script::Raw(raw) => raw.visit(context),
            Script::Source(source) => source.visit(context),
            Script::Var(var) => var.visit(context),
            Script::None => Ok(()),
        }
    }
}

impl Render for Script {
    fn render_script<W: Write>(&self, output: &mut W) -> Result<(), VisitorError> {
        match self {
            Script::Alias(alias) => alias.render_script(output),
            Script::Eval(eval) => eval.render_script(output),
            Script::Export(export) => export.render_script(output),
            Script::Function(function) => function.render_script(output),
            Script::Raw(raw) => raw.render_script(output),
            Script::Source(source) => source.render_script(output),
            Script::Var(var) => var.render_script(output),
            Script::None => Ok(()),
        }?;
        Ok(())
    }
}

impl Visitor for Scripts {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        for script in &self.0 {
            script.visit(context)?;
        }
        Ok(())
    }
}
