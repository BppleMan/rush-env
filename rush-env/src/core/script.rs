use crate::core::script::alias::AliasScript;
use crate::core::script::eval::EvalScript;
use crate::core::script::export::ExportScript;
use crate::core::script::function::FunctionScript;
use crate::core::script::raw::RawScript;
use crate::core::script::source::SourceScript;
use crate::core::script::var::VarScript;
use crate::visitor::{Visit, Visitor, VisitorError};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;

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

impl Visit for Script {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        match self {
            Script::Alias(alias) => alias.visit(context, writer),
            Script::Eval(eval) => eval.visit(context, writer),
            Script::Export(export) => export.visit(context, writer),
            Script::Function(function) => function.visit(context, writer),
            Script::Raw(raw) => raw.visit(context, writer),
            Script::Source(source) => match source.visit(context, writer) {
                Ok(()) => Ok(()),
                Err(VisitorError::SourceFileNotExist(file)) => {
                    warn!("Source file {} does not exist", file);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Script::Var(var) => var.visit(context, writer),
            Script::None => Ok(()),
        }
    }
}

impl Visit for Scripts {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        for script in &self.0 {
            script.visit(context, writer)?;
        }
        Ok(())
    }
}
