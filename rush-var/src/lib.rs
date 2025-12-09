//! # rush-var —— Bash 风格/Posix 风格环境变量插值库
//!
//! 支持 `$VAR`、`${VAR}`、`${VAR:-default}`、`${VAR-default}`、`${VAR:+alt}`、`${VAR+alt}`、`$$` 等常见参数展开写法，
//! 适配多种环境变量源（HashMap/BTreeMap/切片/闭包/链式/系统环境等）。
//!
//! ## 用法示例
//!
//! ```rust
//! use rush_var::expand_var;
//! let env = [("FOO", "bar")];
//! assert_eq!(expand_var("Hello $FOO!", &env), "Hello bar!");
//! assert_eq!(expand_var("path=${BAR:-/usr/local}/bin", &env), "path=/usr/local/bin");
//! assert_eq!(expand_var("alt=${FOO:+y}", &env), "alt=y");
//! ```
//!
//! ## 自定义环境源
//!
//! ```rust
//! use rush_var::env_source::FnVarSrc;
//! use rush_var::expand_var;
//! let env = FnVarSrc(|k: &str| if k == "USER" { Some("alice".to_string()) } else { None });
//! assert_eq!(expand_var("hi_$USER", &env), "hi_alice");
//! ```
//!
//! ## 链式变量源（优先主源，后备源）
//!
//! ```rust
//! use rush_var::env_source::VarSrcChain;
//! use rush_var::expand_var;
//! let main = [("A", "x")];
//! let mut fallback = std::collections::HashMap::new();
//! fallback.insert("B".to_string(), "y".to_string());
//! let chain = VarSrcChain { primary: &main[..], fallback: &fallback };
//! assert_eq!(expand_var("$A,$B", &chain), "x,y");
//! ```

pub mod env_source;
mod expander;

pub use env_source::{EnvSource, EnvSourceChain, FnEnvSource, FnVarSrc, VarSrc, VarSrcChain};
use expander::restore_literal_dollars;

const MAX_EXPAND_DEPTH: usize = 8;

/// Bash/Posix 风格环境变量插值主函数。
///
/// 支持 `$VAR`、`${VAR}`、`${VAR:-default}`（unset/空值使用默认）、`${VAR-default}`（仅 unset 使用默认）、
/// `${VAR:+alt}`（已设置且非空时使用 alt）、`${VAR+alt}`（只要设置就使用 alt）以及 `$$`（字面 `$`）。
/// 默认值/备选值中的 `word` 会再做一轮参数展开（单次），整体最多递归 8 层，避免无限自引用。
pub fn expand_var(input: &str, env: &impl VarSrc) -> String {
    expand_recursive(input, env)
}

/// 便捷函数：直接使用进程环境变量进行插值。
pub fn expand_var_env(input: &str) -> String {
    let vars = std::env::vars();
    expand_var(input, &vars)
}

fn expand_recursive(input: &str, env: &impl VarSrc) -> String {
    let mut current = input.to_string();
    for _ in 0..MAX_EXPAND_DEPTH {
        let expanded = expand_once(&current, env);
        if expanded == current || !expanded.contains('$') {
            return restore_literal_dollars(expanded);
        }
        current = expanded;
    }
    restore_literal_dollars(current)
}

fn expand_once(input: &str, env: &impl VarSrc) -> String {
    expander::expand(input, env)
}
