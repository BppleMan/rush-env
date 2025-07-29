use crate::visitor::{Visit, Visitor, VisitorError};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Path(pub String);

#[derive(Default, Debug, Clone, Serialize)]
#[derive(AsRef, AsMut, Deref, DerefMut)]
#[serde(rename_all = "snake_case")]
pub struct Paths(pub Vec<Path>);

impl Path {
    pub fn tag() -> &'static str {
        "<path>"
    }

    fn export(&self) -> Result<(), VisitorError> {
        if let Some(path) = std::env::var_os("PATH") {
            let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
            paths.push(PathBuf::from(&self.0));
            let new_path = std::env::join_paths(paths)?;
            unsafe {
                std::env::set_var("PATH", &new_path);
            }
        }
        Ok(())
    }
}

impl Paths {
    fn export(&self) -> Result<(), VisitorError> {
        if let Some(path) = std::env::var_os("PATH") {
            let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
            paths.extend(self.0.iter().map(PathBuf::from));
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

impl Visit for Paths {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, _writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        self.export()?;
        context.paths.extend(self.0.iter());
        Ok(())
    }
}

impl AsRef<OsStr> for Path {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl<'de> Deserialize<'de> for Paths {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct List {
            #[serde(rename = "$value", default)]
            element: Vec<Path>,
        }
        Ok(Paths(List::deserialize(deserializer)?.element))
    }
}
