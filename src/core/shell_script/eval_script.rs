use crate::core::platform::{ARCH, OS};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EvalScript {
    #[serde(rename = "$text")]
    pub script: String,
    #[serde(rename = "@os", default)]
    pub os: Vec<OS>,
    #[serde(rename = "@arch", default)]
    pub arch: Vec<ARCH>,
}
