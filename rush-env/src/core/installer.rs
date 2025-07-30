use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
#[serde(rename_all = "snake_case")]
pub struct Install {
    #[serde(rename = "$value")]
    pub installer: Installer,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Installer {
    #[default]
    Builtin,
    #[serde(rename = "git")]
    Git(GitInstaller),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GitInstaller {
    pub url: String,
    pub dest: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
}
