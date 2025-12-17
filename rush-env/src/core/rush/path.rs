use crate::core::rush::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Path {
    pub value: String,
    #[serde(default)]
    pub condition: Condition,
}

impl Path {
    pub fn tag() -> &'static str {
        "<path>"
    }

    fn export(&self) -> Result<(), VisitorError> {
        if let Some(path) = std::env::var_os("PATH") {
            let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
            paths.push(PathBuf::from(&self.value));
            let new_path = std::env::join_paths(paths)?;
            unsafe {
                std::env::set_var("PATH", &new_path);
            }
        }
        Ok(())
    }
}

impl Visit for Path {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, _writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        self.export()?;
        context.paths.push(self);
        Ok(())
    }
}

impl AsRef<OsStr> for Path {
    fn as_ref(&self) -> &OsStr {
        self.value.as_ref()
    }
}
