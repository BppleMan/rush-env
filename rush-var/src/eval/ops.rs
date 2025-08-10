use super::ExpansionOptions as Options;
use crate::ast::*;
use crate::env::EnvVars;

use crate::eval::evaluate_expr; // for recursion
use crate::eval::glob_match;
// no direct usage of path::apply_path_modifier in ops module

/// 处理默认/赋值/替代/错误四类操作
pub(crate) fn handle_defaulting_operation<E: EnvVars + ?Sized>(
    inner_result: Result<String, Error>,
    should_use_default: bool,
    op: &DefaultOp,
    word: &[Word],
    env: &E,
    opt: &Options,
) -> Result<String, Error> {
    match op {
        DefaultOp::Dash => {
            if should_use_default {
                evaluate_word_list(word, env, opt)
            } else {
                inner_result
            }
        }
        DefaultOp::Assign => {
            if should_use_default {
                let default_val = evaluate_word_list(word, env, opt)?;
                // TODO: 赋值副作用
                Ok(default_val)
            } else {
                inner_result
            }
        }
        DefaultOp::Plus => {
            if should_use_default {
                Ok(String::new())
            } else {
                evaluate_word_list(word, env, opt)
            }
        }
        DefaultOp::QMark => {
            if should_use_default {
                let msg = evaluate_word_list(word, env, opt)?;
                Err(Error::Eval(msg))
            } else {
                inner_result
            }
        }
    }
}

/// 计算索引（保留原有语义）
pub(crate) fn calculate_array_index(i: i64, len: usize) -> Result<usize, Error> {
    if i == i64::MAX || i == i64::MIN {
        return Err(Error::IndexOutOfBounds(format!("Index out of range: {}", i)));
    }
    let idx = if i < 0 {
        let pos = len as i64 + i;
        if pos < 0 {
            return Ok(usize::MAX);
        }
        pos as usize
    } else {
        if i == 0 {
            return Ok(usize::MAX);
        }
        (i - 1) as usize
    };
    Ok(idx)
}

pub(crate) fn calculate_slice_indices(start: i64, end: i64, len: usize) -> (usize, usize) {
    let start_idx = if start < 0 { 0 } else { ((start - 1) as usize).min(len) };
    let end_idx = if end < 0 { len } else { ((end - 1) as usize).min(len) };
    (start_idx, end_idx)
}

/// 评估 word 列表
pub(crate) fn evaluate_word_list<E: EnvVars + ?Sized>(words: &[Word], env: &E, opt: &Options) -> Result<String, Error> {
    let mut result = String::new();
    for word in words {
        match word {
            Word::Text(text) => result.push_str(text),
            Word::Param(expr) => {
                let val = evaluate_expr(expr, env, opt)?;
                result.push_str(&val);
            }
            Word::CmdSubst(cmd) => {
                if opt.allow_exec_subst {
                    return Err(Error::Unsupported("command substitution not implemented".to_string()));
                } else {
                    result.push_str(&format!("$({}", cmd));
                    result.push(')');
                }
            }
            Word::ArithSubst(expr) => {
                result.push_str(&format!("$(({})", expr));
                result.push(')');
            }
        }
    }
    Ok(result)
}

// New clearer name; keep old function for backward compatibility
pub(crate) fn evaluate_words<E: EnvVars + ?Sized>(words: &[Word], env: &E, opt: &Options) -> Result<String, Error> {
    evaluate_word_list(words, env, opt)
}

pub(crate) fn remove_prefix(value: &str, pattern: &str, long: bool) -> Result<String, Error> {
    if long {
        let mut best = 0;
        for i in 1..=value.len() {
            let prefix = &value[..i];
            if glob_match(pattern, prefix) {
                best = i;
            }
        }
        Ok(value[best..].to_string())
    } else {
        for i in 1..=value.len() {
            let prefix = &value[..i];
            if glob_match(pattern, prefix) {
                return Ok(value[i..].to_string());
            }
        }
        Ok(value.to_string())
    }
}

pub(crate) fn remove_suffix(value: &str, pattern: &str, long: bool) -> Result<String, Error> {
    let mut positions = Vec::new();
    for i in 0..=value.len() {
        let suffix = &value[i..];
        if glob_match(pattern, suffix) {
            positions.push(i);
        }
    }
    if positions.is_empty() {
        return Ok(value.to_string());
    }
    if long {
        Ok(value[..positions[0]].to_string())
    } else {
        Ok(value[..*positions.last().unwrap()].to_string())
    }
}

pub(crate) fn replace_pattern(value: &str, pattern: &str, replacement: &str, scope: &ReplaceScope) -> Result<String, Error> {
    match scope {
        ReplaceScope::First => {
            if let Some(pos) = find_pattern_match(value, pattern) {
                let mut r = String::new();
                r.push_str(&value[..pos.0]);
                r.push_str(replacement);
                r.push_str(&value[pos.1..]);
                Ok(r)
            } else {
                Ok(value.to_string())
            }
        }
        ReplaceScope::Global => {
            let mut r = value.to_string();
            let mut m = find_all_pattern_matches(value, pattern);
            m.reverse();
            for (s, e) in m {
                r.replace_range(s..e, replacement);
            }
            Ok(r)
        }
        ReplaceScope::AnchorPrefix => {
            if let Some(stripped) = value.strip_prefix(pattern) {
                Ok(format!("{}{}", replacement, stripped))
            } else {
                Ok(value.to_string())
            }
        }
        ReplaceScope::AnchorSuffix => {
            if let Some(stripped) = value.strip_suffix(pattern) {
                Ok(format!("{}{}", stripped, replacement))
            } else {
                Ok(value.to_string())
            }
        }
    }
}

pub(crate) fn substring(value: &str, offset: i64, len: Option<i64>) -> Result<String, Error> {
    let chars: Vec<char> = value.chars().collect();
    let total_len = chars.len() as i64;
    let start_pos = if offset < 0 {
        (total_len + offset).max(0) as usize
    } else {
        offset as usize
    };
    if start_pos >= chars.len() {
        return Ok(String::new());
    }
    let end_pos = if let Some(l) = len {
        if l < 0 {
            chars.len()
        } else {
            (start_pos + l as usize).min(chars.len())
        }
    } else {
        chars.len()
    };
    Ok(chars[start_pos..end_pos].iter().collect())
}

pub(crate) fn find_pattern_match(text: &str, pattern: &str) -> Option<(usize, usize)> {
    text.find(pattern).map(|pos| (pos, pos + pattern.len()))
}

pub(crate) fn find_all_pattern_matches(text: &str, pattern: &str) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    if pattern.is_empty() {
        return matches;
    }
    let mut start = 0;
    while start < text.len() {
        if let Some(pos) = text[start..].find(pattern) {
            let abs = start + pos;
            matches.push((abs, abs + pattern.len()));
            start = abs + 1;
        } else {
            break;
        }
    }
    matches
}
