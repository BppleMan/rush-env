use crate::core::rush::platform::Platform;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    All {
        conditions: Vec<Condition>,
    },
    Any {
        conditions: Vec<Condition>,
    },
    Not {
        condition: Box<Condition>,
    },
    Has {
        command: String,
    },
    FileExists {
        path: String,
    },
    DirExists {
        path: String,
    },
    LinkExists {
        path: String,
    },
    Platform(Platform),
    #[default]
    None,
}

impl Condition {
    pub fn check(&self) -> bool {
        match self {
            Condition::All { conditions } => conditions.iter().all(Condition::check),
            Condition::Any { conditions } => conditions.iter().any(Condition::check),
            Condition::Not { condition } => !condition.check(),
            Condition::Has { command } => Self::has_command(command),
            Condition::FileExists { path } => Self::file_exists(path),
            Condition::DirExists { path } => Self::dir_exists(path),
            Condition::LinkExists { path } => Self::link_exists(path),
            Condition::Platform(platform) => platform.contains_current(),
            Condition::None => true,
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
