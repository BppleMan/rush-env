use crate::core::shell_script::alias_script::AliasScript;
use crate::core::shell_script::eval_script::EvalScript;
use crate::core::shell_script::export_script::ExportScript;
use crate::core::shell_script::function_script::FunctionScript;
use crate::core::shell_script::raw_script::RawScript;
use crate::core::shell_script::source_script::SourceScript;
use crate::core::shell_script::var_script::VarScript;
use serde::{Deserialize, Serialize};

pub mod alias_script;
pub mod eval_script;
pub mod export_script;
pub mod function_script;
pub mod raw_script;
pub mod source_script;
pub mod var_script;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellScript {
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
