//! # rush-var —— Bash风格环境变量插值库
//!
//! 支持 $VAR, ${VAR}, ${VAR:-default}，适配多种环境变量源（HashMap/BTreeMap/切片/闭包/链式/系统环境等）。
//!
//! ## 用法示例
//!
//! ```rust
//! use rush_var::expand_env;
//! let env = [ ("FOO", "bar") ];
//! assert_eq!(expand_env("Hello $FOO!", &env), "Hello bar!");
//! assert_eq!(expand_env("path=${BAR:-/usr/local}/bin", &env), "path=/usr/local/bin");
//! ```
//!
//! ## 支持自定义环境源
//!
//! ```rust
//! use rush_var::env_source::{FnEnvSource};
//! use rush_var::expand_env;
//! let env = FnEnvSource(|k: &str| if k == "USER" { Some("alice".to_string()) } else { None });
//! assert_eq!(expand_env("hi_$USER", &env), "hi_alice");
//! ```
//!
//! ## 支持链式变量源（优先主源，后备源）
//!
//! ```rust
//! use rush_var::env_source::{EnvSourceChain};
//! use rush_var::expand_env;
//! let main = [ ("A", "x") ];
//! let mut fallback = std::collections::HashMap::new();
//! fallback.insert("B".to_string(), "y".to_string());
//! let chain = EnvSourceChain { primary: &main[..], fallback: &fallback };
//! assert_eq!(expand_env("$A,$B", &chain), "x,y");
//! ```

pub mod env_source;

use crate::env_source::EnvSource;

pub fn expand_env_vars(input: &str) -> String {
    let vars = std::env::vars();
    expand_env_recursive(input, &vars)
}

pub fn expand_env_recursive(input: &str, env: &impl EnvSource) -> String {
    const MAX_EXPAND_DEPTH: usize = 8;
    fn inner(s: &str, env: &impl EnvSource, depth: usize) -> String {
        if depth >= MAX_EXPAND_DEPTH {
            return s.to_string();
        }
        let expanded = expand_env(s, env);
        if expanded.contains('$') && expanded != s {
            inner(&expanded, env, depth + 1)
        } else {
            expanded
        }
    }
    inner(input, env, 0)
}

/// Bash 风格环境变量插值主函数。
///
/// 支持 $VAR、${VAR}、${VAR:-default}、$$（字面$），适配多种环境变量源。
///
/// # 用法示例
/// ```rust
/// use rush_var::expand_env;
/// let env = [ ("FOO", "bar") ];
/// assert_eq!(expand_env("$FOO/bin", &env), "bar/bin");
/// assert_eq!(expand_env("${BAR:-default}/lib", &env), "default/lib");
/// ```
pub fn expand_env(input: &str, env: &impl EnvSource) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('$') => {
                    chars.next(); // consume second $
                    result.push('$');
                }
                Some('{') => {
                    chars.next(); // consume '{'
                    let mut key = String::new();
                    let mut default = None;
                    let mut in_default = false;
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' {
                            chars.next(); // consume '}'
                            break;
                        } else if ch == ':' && chars.clone().nth(1) == Some('-') {
                            chars.next();
                            chars.next(); // consume :-
                            in_default = true;
                        } else {
                            if in_default {
                                default.get_or_insert(String::new()).push(ch);
                            } else {
                                key.push(ch);
                            }
                            chars.next();
                        }
                    }
                    let val = env.get(&key).or(default.as_ref().cloned()).unwrap_or_default();
                    result.push_str(&val);
                }
                Some(ch) if ch.is_alphanumeric() || *ch == '_' => {
                    let mut key = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            key.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let val = env.get(&key).unwrap_or_default();
                    result.push_str(&val);
                }
                _ => {
                    result.push('$');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env_source::{EnvSourceChain, FnEnvSource};
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_expand_basic() {
        let mut env = HashMap::new();
        env.insert("FOO".into(), "bar".into());
        assert_eq!(expand_env("$FOO/bin", &env), "bar/bin");
    }

    #[test]
    fn test_expand_brace() {
        let mut env = HashMap::new();
        env.insert("FOO".into(), "bar".into());
        assert_eq!(expand_env("${FOO}/lib", &env), "bar/lib");
    }

    #[test]
    fn test_expand_with_default() {
        let env = HashMap::new();
        assert_eq!(expand_env("${FOO:-baz}/bin", &env), "baz/bin");
    }

    #[test]
    fn test_unterminated_brace() {
        let mut env = HashMap::new();
        env.insert("FOO".into(), "bar".into());
        assert_eq!(expand_env("${FOO", &env), "bar");
    }

    #[test]
    fn test_mixed_vars_and_defaults() {
        let mut env = HashMap::new();
        env.insert("X".into(), "123".into());
        env.insert("Y".into(), "abc".into());
        assert_eq!(expand_env("$X/${Y:-zzz}/$Z", &env), "123/abc/");
    }

    #[test]
    fn test_literal_dollar_sign() {
        let env = HashMap::new();
        assert_eq!(expand_env("Price is $$100", &env), "Price is $100");
    }

    #[test]
    fn test_non_alphanumeric_after_dollar() {
        let env = HashMap::new();
        assert_eq!(expand_env("Hello $!", &env), "Hello $!");
    }

    #[test]
    fn test_multiple_variables() {
        let mut env = HashMap::new();
        env.insert("A".into(), "1".into());
        env.insert("B".into(), "2".into());
        env.insert("C".into(), "3".into());
        assert_eq!(expand_env("$A-$B-${C:-0}", &env), "1-2-3");
    }

    #[test]
    fn test_empty_input() {
        let env = HashMap::new();
        assert_eq!(expand_env("", &env), "");
    }

    #[test]
    fn test_default_value_with_special_chars() {
        let env = HashMap::new();
        assert_eq!(expand_env("${MISSING:-/usr/local/bin}", &env), "/usr/local/bin");
    }

    #[test]
    fn test_no_substitution() {
        let env = HashMap::new();
        assert_eq!(expand_env("just a string", &env), "just a string");
    }

    #[test]
    fn test_env_source_btree_map() {
        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "baz".into());
        assert_eq!(expand_env("$FOO", &env), "baz");
    }

    #[test]
    fn test_env_source_slice() {
        let env: &[(&str, &str)] = &[("FOO", "baz")];
        assert_eq!(expand_env("prefix_$FOO", &env), "prefix_baz");
    }

    #[test]
    fn test_env_source_fn_adapter() {
        let env_fn = FnEnvSource(|key: &str| if key == "FOO" { Some("baz".into()) } else { None });
        assert_eq!(expand_env("abc$FOO", &env_fn), "abcbaz");
    }

    #[test]
    fn test_chain_env_source() {
        let env1 = [("FOO", "a")];
        let mut env2 = HashMap::new();
        env2.insert("BAR".into(), "b".into());
        let chain = EnvSourceChain {
            primary: &env1[..],
            fallback: &env2,
        };
        assert_eq!(expand_env("$FOO:$BAR:$BAZ", &chain), "a:b:");
    }

    #[test]
    fn test_recursive_expand() {
        let mut env = HashMap::new();
        env.insert("FOO".into(), "$BAR".into());
        env.insert("BAR".into(), "hello".into());
        assert_eq!(expand_env_recursive("$FOO world", &env), "hello world");
    }

    #[test]
    fn test_recursive_multi_layer() {
        let mut env = HashMap::new();
        env.insert("A".into(), "$B".into());
        env.insert("B".into(), "$C".into());
        env.insert("C".into(), "$D".into());
        env.insert("D".into(), "42".into());
        assert_eq!(expand_env_recursive("A=$A, B=$B, C=$C, D=$D", &env), "A=42, B=42, C=42, D=42");
    }

    #[test]
    fn test_recursive_prevent_infinite() {
        let mut env = HashMap::new();
        env.insert("LOOP".into(), "$LOOP".into());
        let res = expand_env_recursive("start:$LOOP:end", &env);
        // 最多递归8次，最后返回原样
        assert!(res.contains("$LOOP"));
    }
}
