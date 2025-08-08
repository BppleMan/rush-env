//! # rush-var —— Shell 风格环境变量插值库
//!
//! 支持 POSIX/Bash 风格的变量展开语法，适配多种环境变量源（`HashMap`、`BTreeMap`、切片、闭包、链式、系统环境等）。
//!
//! 目前实现了如下语法特性：
//!
//! - `$VAR`、`${VAR}` 基本取值
//! - 默认值/赋值/条件值/报错：`${VAR:-word}`、`${VAR=word}` 等
//! - 取长度：`${#VAR}`
//! - 删除前缀/后缀：`${VAR#pat}`、`${VAR%pat}` 及双号版本
//! - 子串截取：`${VAR:offset}`、`${VAR:offset:length}`
//! - 模式替换：`${VAR/pat/repl}`、`${VAR//pat/repl}`、`${VAR/#pat/repl}`、`${VAR/%pat/repl}`
//! - 间接展开：`${!VAR}`
//!
//! 解析并不完全等同于真实 shell，模式匹配部分以字面字符串实现，不支持通配符。

pub mod env_source;

use crate::env_source::EnvSource;

/// 从当前进程环境变量中展开字符串。
pub fn expand_env_vars(input: &str) -> String {
    let vars = std::env::vars();
    expand_env_recursive(input, &vars)
}

/// 递归展开变量，最多递归 8 层，防止无限循环。
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
/// 支持 `$VAR`、`${VAR}` 及一系列扩展语法（见模块文档）。
pub fn expand_env(input: &str, env: &impl EnvSource) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('$') => {
                    chars.next();
                    result.push('$');
                }
                Some('{') => {
                    chars.next(); // consume '{'
                    let mut expr = String::new();
                    let mut depth = 1;
                    while let Some(ch) = chars.next() {
                        match ch {
                            '{' => {
                                depth += 1;
                                expr.push(ch);
                            }
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                } else {
                                    expr.push(ch);
                                }
                            }
                            _ => expr.push(ch),
                        }
                    }
                    result.push_str(&eval_braced(&expr, env));
                }
                Some(ch) if is_var_char(*ch) || ch.is_ascii_digit() => {
                    let mut name = String::new();
                    while let Some(&ch) = chars.peek() {
                        if is_var_char(ch) || ch.is_ascii_digit() {
                            name.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    result.push_str(&env.get(&name).unwrap_or_default());
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

fn is_var_char(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

fn eval_braced(expr: &str, env: &impl EnvSource) -> String {
    if let Some(rest) = expr.strip_prefix('#') {
        let val = env.get(rest).unwrap_or_default();
        return val.chars().count().to_string();
    }
    if let Some(rest) = expr.strip_prefix('!') {
        let key = env.get(rest).unwrap_or_default();
        return env.get(&key).unwrap_or_default();
    }

    // 解析变量名
    let mut end = 0;
    for (i, ch) in expr.char_indices() {
        if is_var_char(ch) || ch.is_ascii_digit() {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    let name = &expr[..end];
    let rest = &expr[end..];
    let val_opt = env.get(name);

    // 默认值/赋值/条件值/报错
    if let Some((colon, op, word)) = parse_colon_op(rest) {
        let is_set = val_opt.as_ref().map(|v| !v.is_empty()).unwrap_or(false);
        let cond = if colon { is_set } else { val_opt.is_some() };
        match op {
            '-' => {
                if cond {
                    return val_opt.unwrap();
                } else {
                    return word.to_string();
                }
            }
            '=' => {
                if cond {
                    return val_opt.unwrap();
                } else {
                    return word.to_string();
                }
            }
            '+' => {
                if cond {
                    return word.to_string();
                } else {
                    return String::new();
                }
            }
            '?' => {
                if cond {
                    return val_opt.unwrap();
                } else {
                    return word.to_string();
                }
            }
            _ => {}
        }
    }

    // 子串截取
    if let Some(rest2) = rest.strip_prefix(':') {
        return substring(val_opt.unwrap_or_default(), rest2);
    }

    // 前缀/后缀删除
    if let Some(pat) = rest.strip_prefix("##") {
        let v = val_opt.unwrap_or_default();
        return remove_prefix(&v, pat);
    }
    if let Some(pat) = rest.strip_prefix('#') {
        let v = val_opt.unwrap_or_default();
        return remove_prefix(&v, pat);
    }
    if let Some(pat) = rest.strip_prefix("%%") {
        let v = val_opt.unwrap_or_default();
        return remove_suffix(&v, pat);
    }
    if let Some(pat) = rest.strip_prefix('%') {
        let v = val_opt.unwrap_or_default();
        return remove_suffix(&v, pat);
    }

    // 模式替换（需先匹配双斜杠）
    if let Some(repl_spec) = rest.strip_prefix("//") {
        return replace_pattern(val_opt.unwrap_or_default().as_str(), repl_spec, true);
    }
    if let Some(repl_spec) = rest.strip_prefix('/') {
        return replace_pattern(val_opt.unwrap_or_default().as_str(), repl_spec, false);
    }

    val_opt.unwrap_or_default()
}

fn parse_colon_op(rest: &str) -> Option<(bool, char, &str)> {
    let bytes = rest.as_bytes();
    if rest.len() >= 2 && bytes[0] == b':' {
        let op = bytes[1] as char;
        if "-+=?".contains(op) {
            return Some((true, op, &rest[2..]));
        }
    } else if !rest.is_empty() {
        let op = bytes[0] as char;
        if "-+=?".contains(op) {
            return Some((false, op, &rest[1..]));
        }
    }
    None
}

fn substring(val: String, spec: &str) -> String {
    let mut parts = spec.splitn(2, ':');
    let off_str = parts.next().unwrap_or("0");
    let len_str = parts.next();
    let chars: Vec<char> = val.chars().collect();
    let len_val = chars.len() as isize;
    let mut off: isize = off_str.trim().parse().unwrap_or(0);
    if off < 0 {
        off = len_val + off;
    }
    let off = off.clamp(0, len_val) as usize;
    if let Some(len_s) = len_str {
        let n: isize = len_s.trim().parse().unwrap_or(0);
        if n < 0 {
            return String::new();
        }
        let end = (off as isize + n).clamp(0, len_val) as usize;
        chars[off..end].iter().collect()
    } else {
        chars[off..].iter().collect()
    }
}

fn remove_prefix(val: &str, pat: &str) -> String {
    if let Some(rest) = val.strip_prefix(pat) {
        rest.to_string()
    } else {
        val.to_string()
    }
}

fn remove_suffix(val: &str, pat: &str) -> String {
    if let Some(rest) = val.strip_suffix(pat) {
        rest.to_string()
    } else {
        val.to_string()
    }
}

fn replace_pattern(val: &str, spec: &str, all: bool) -> String {
    let mut parts = spec.splitn(2, '/');
    let mut pat = parts.next().unwrap_or("");
    let repl = parts.next().unwrap_or("");
    let anchor_start = pat.starts_with('#');
    let anchor_end = pat.starts_with('%');
    if anchor_start || anchor_end {
        pat = &pat[1..];
    }
    if pat.is_empty() {
        return val.to_string();
    }
    if anchor_start && val.starts_with(pat) {
        return format!("{}{}", repl, &val[pat.len()..]);
    }
    if anchor_end && val.ends_with(pat) {
        return format!("{}{}", &val[..val.len() - pat.len()], repl);
    }
    if all { val.replace(pat, repl) } else { val.replacen(pat, repl, 1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env_source::{EnvSourceChain, FnEnvSource};
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_basic_and_default() {
        let mut env = HashMap::new();
        env.insert("FOO".into(), "bar".into());
        assert_eq!(expand_env("$FOO/${BAR:-baz}", &env), "bar/baz");
    }

    #[test]
    fn test_length_and_remove() {
        let mut env = HashMap::new();
        env.insert("NAME".into(), "/usr/bin".into());
        assert_eq!(expand_env("len=${#NAME}", &env), "len=8");
        assert_eq!(expand_env("${NAME#/usr}", &env), "/bin");
        assert_eq!(expand_env("${NAME%bin}", &env), "/usr/");
    }

    #[test]
    fn test_substring_and_replace() {
        let mut env = HashMap::new();
        env.insert("WORD".into(), "helloworld".into());
        assert_eq!(expand_env("${WORD:5}", &env), "world");
        assert_eq!(expand_env("${WORD:0:5}", &env), "hello");
        assert_eq!(expand_env("${WORD/hello/hi}", &env), "hiworld");
        assert_eq!(expand_env("${WORD//l/_}", &env), "he__owor_d");
        assert_eq!(expand_env("${WORD/#hello/hi}", &env), "hiworld");
        assert_eq!(expand_env("${WORD/%world/earth}", &env), "helloearth");
    }

    #[test]
    fn test_indirect_and_recursive() {
        let mut env = HashMap::new();
        env.insert("A".into(), "B".into());
        env.insert("B".into(), "C".into());
        env.insert("C".into(), "ok".into());
        assert_eq!(expand_env("${!A}", &env), "C");
        env.insert("VAR".into(), "$C".into());
        assert_eq!(expand_env_recursive("$VAR", &env), "ok");
    }

    #[test]
    fn test_env_sources() {
        let env_slice: &[(&str, &str)] = &[("FOO", "x")];
        assert_eq!(expand_env("$FOO", &env_slice), "x");
        let mut map = BTreeMap::new();
        map.insert("BAR".into(), "y".into());
        assert_eq!(expand_env("$BAR", &map), "y");
        let func = FnEnvSource(|k: &str| if k == "Z" { Some("z".into()) } else { None });
        assert_eq!(expand_env("$Z", &func), "z");
        let chain = EnvSourceChain {
            primary: &env_slice[..],
            fallback: &map,
        };
        assert_eq!(expand_env("$FOO:$BAR", &chain), "x:y");
    }
}
