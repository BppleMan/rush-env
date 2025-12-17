use crate::error::RushContextError;
use crate::error::RushContextError::EnvVarNotFound;
use indexmap::IndexMap;
use rush_var::VarSrc;
use std::collections::VecDeque;
use std::path::PathBuf;
use which::which_in;

pub struct RushContext {
    cwd: PathBuf,
    vars: IndexMap<String, String>,
    paths: VecDeque<String>,
}

impl RushContext {
    pub fn from_env() -> Result<Self, RushContextError> {
        let cwd = std::env::current_dir().map_err(|e| EnvVarNotFound(format!("CWD: {}", e)))?;
        let vars = std::env::vars().collect::<IndexMap<_, _>>();
        let paths = std::env::var("PATH")
            .map(|path| path.split(':').map(|s| s.to_string()).collect::<VecDeque<_>>())
            .map_err(|_| EnvVarNotFound("PATH".to_string()))?;
        Ok(Self { cwd, vars, paths })
    }

    pub fn push_path(&mut self, path: impl AsRef<str>) {
        self.paths.push_front(path.as_ref().to_string());
    }

    pub fn has_path(&self, path: impl AsRef<str>) -> bool {
        self.paths.iter().any(|p| p == path.as_ref())
    }

    pub fn push_var(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) {
        self.vars.insert(key.as_ref().to_string(), value.as_ref().to_string());
    }

    pub fn get_var(&self, key: impl AsRef<str>) -> Option<&String> {
        self.vars.get(key.as_ref())
    }

    pub fn which(&self, command: impl AsRef<str>) -> Option<PathBuf> {
        let paths = self.paths.iter().filter(|s| !s.is_empty()).fold(String::new(), |mut acc, dir| {
            acc.push(':');
            acc.push_str(dir);
            acc
        });
        which_in(command.as_ref(), Some(paths), &self.cwd).ok()
    }
}

impl VarSrc for RushContext {
    fn get(&self, key: &str) -> Option<String> {
        self.get_var(key).cloned()
    }
}
