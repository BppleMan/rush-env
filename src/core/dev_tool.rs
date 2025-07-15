use crate::core::platform::{ARCH, OS};
use crate::core::shell_script::ShellScript;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DevTool {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@bin_path", default)]
    pub bin_path: Option<String>,
    #[serde(rename = "$value", default)]
    pub scripts: Vec<ShellScript>,
    #[serde(rename = "@os", default)]
    pub os: Vec<OS>,
    #[serde(rename = "@arch", default)]
    pub arch: Vec<ARCH>,
}
