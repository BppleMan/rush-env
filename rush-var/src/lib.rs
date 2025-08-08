//! Parameter expansion library with a simplified zsh-first semantics.

mod ast;
mod env;
mod eval;
mod lexer;
mod parser;

pub use ast::{Error, ParamExpr};
pub use env::{Env, Value};
pub use eval::{expand_str, Mode, Options};
pub use parser::parse_braced;

/// Expand using current process environment variables.
pub fn expand_env_vars(input: &str) -> String {
    let mut env = Env::new();
    for (k, v) in std::env::vars() {
        env.set_scalar(k, v);
    }
    expand_str(input, &env, &Options::default()).unwrap_or_default()
}

/// Expand using a provided environment map.
pub fn expand_env_recursive(input: &str, env: &Env) -> String {
    expand_str(input, env, &Options::default()).unwrap_or_default()
}
