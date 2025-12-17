use thiserror::Error;

#[derive(Debug, Error)]
pub enum RushContextError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}
