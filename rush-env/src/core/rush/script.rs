use crate::core::rush::script::alias::AliasScript;
use crate::core::rush::script::eval::EvalScript;
use crate::core::rush::script::export::ExportScript;
use crate::core::rush::script::function::FunctionScript;
use crate::core::rush::script::raw::RawScript;
use crate::core::rush::script::source::SourceScript;
use crate::core::rush::script::var::VarScript;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};
use tracing::warn;

pub mod alias;
pub mod eval;
pub mod export;
pub mod function;
pub mod raw;
pub mod source;
pub mod var;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Script {
    Alias(AliasScript),
    Eval(EvalScript),
    Export(ExportScript),
    Function(FunctionScript),
    Raw(RawScript),
    Source(SourceScript),
    Var(VarScript),
    #[default]
    None,
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
