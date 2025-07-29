use crate::core::platform::Platform;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, Serialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
#[serde(rename_all = "snake_case")]
pub struct Condition(#[serde(rename = "$value")] Predicate);

#[derive(Default, Debug, Clone, Serialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
pub struct Conditions(Vec<Predicate>);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    #[serde(rename = "all")]
    All(Conditions),
    #[serde(rename = "any")]
    Any(Conditions),
    #[serde(rename = "not")]
    Not(Box<Condition>),
    #[serde(rename = "has")]
    Has(String),
    #[serde(rename = "file_exists")]
    FileExists(String),
    #[serde(rename = "dir_exists")]
    DirExists(String),
    #[serde(rename = "link_exists")]
    LinkExists(String),
    #[serde(rename = "platform")]
    Platform(Platform),
    #[default]
    None,
}

impl Predicate {
    pub fn check(&self) -> bool {
        match self {
            Predicate::All(conditions) => conditions.iter().all(Predicate::check),
            Predicate::Any(conditions) => conditions.iter().any(Predicate::check),
            Predicate::Not(condition) => !condition.check(),
            Predicate::Has(command) => Self::has_command(command),
            Predicate::FileExists(path) => Self::file_exists(path),
            Predicate::DirExists(path) => Self::dir_exists(path),
            Predicate::LinkExists(path) => Self::link_exists(path),
            Predicate::Platform(platform) => platform.contains_current(),
            Predicate::None => true,
        }
    }

    fn has_command(command: &str) -> bool {
        which::which(command).is_ok()
    }

    fn file_exists(path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }

    fn dir_exists(path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }

    fn link_exists(path: &str) -> bool {
        std::path::Path::new(path).is_symlink()
    }
}

impl Condition {
    pub fn check(&self) -> bool {
        self.0.check()
    }
}

impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SinglePredicate {
            #[serde(rename = "$value")]
            predicate: Predicate,
        }
        Ok(Condition(SinglePredicate::deserialize(deserializer)?.predicate))
    }
}

impl<'de> Deserialize<'de> for Conditions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct List {
            #[serde(rename = "$value", default)]
            element: Vec<Predicate>,
        }
        Ok(Conditions(List::deserialize(deserializer)?.element))
    }
}
