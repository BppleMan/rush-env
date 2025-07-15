use crate::core::platform::{ARCH, OS};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SourceScript {
    #[serde(rename = "$text")]
    pub file: String,
    #[serde(rename = "@os", default)]
    pub os: Vec<OS>,
    #[serde(rename = "@arch", default)]
    pub arch: Vec<ARCH>,
}
