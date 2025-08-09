use crate::ast::*;
use crate::env::{Env, Value};
use std::path::Path;

/// Configuration options for expansion behavior
#[derive(Debug, Clone)]
pub struct Options {
    pub mode: Mode,
    pub allow_exec_subst: bool, // Allow $(command) execution
    pub allow_flag_e: bool,     // Allow (e) flag for re-expansion
    pub glob_impl: GlobKind,    // Glob matching implementation
}

/// Shell mode for compatibility
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Zsh,
    Bash,
    Posix,
    Union, // Support all features
}

/// Glob matching implementation
#[derive(Debug, Clone, PartialEq)]
pub enum GlobKind {
    Simple, // Basic *, ?, [..] matching
}

impl Default for Options {
    fn default() -> Self {
        Self {
            mode: Mode::Zsh,
            allow_exec_subst: false,
            allow_flag_e: false,
            glob_impl: GlobKind::Simple,
        }
    }
}

/// Main expansion function - expands all parameter expansions in input string
pub fn expand_str(input: &str, env: &Env, opt: &Options) -> Result<String, Error> {
    let expansions = crate::parser::find_expansions(input)?;

    if expansions.is_empty() {
        return Ok(input.to_string());
    }

    let mut result = String::new();
    let mut last_end = 0;

    for (start, end, expr) in expansions {
        // Add text before expansion
        result.push_str(&input[last_end..start]);

        // Evaluate expansion
        let expanded = evaluate_expr(&expr, env, opt)?;
        result.push_str(&expanded);

        last_end = end;
    }

    // Add remaining text
    result.push_str(&input[last_end..]);

    Ok(result)
}

/// Evaluate a parameter expression
pub fn evaluate_expr(expr: &ParamExpr, env: &Env, opt: &Options) -> Result<String, Error> {
    match expr {
        ParamExpr::Ref { target, index } => {
            let value = get_target_value(target, env)?;
            apply_index(&value, index)
        }

        ParamExpr::Length { inner } => {
            // For length operation with arrays, we need special handling
            // ${#ARR} should return the length of the first element, not the joined string
            if let ParamExpr::Ref { target, index } = inner.as_ref() {
                if let Index::None = index {
                    let value = get_target_value(target, env)?;
                    match &value {
                        Value::Array(arr) => {
                            // For arrays without index, return length of first element
                            let first_len = arr.first().map(|s| s.len()).unwrap_or(0);
                            Ok(first_len.to_string())
                        }
                        Value::Assoc(map) => {
                            // For associative arrays, return number of keys
                            Ok(map.len().to_string())
                        }
                        Value::Scalar(s) => Ok(s.len().to_string()),
                    }
                } else {
                    // For indexed access, evaluate normally and get string length
                    let result = apply_index(&get_target_value(target, env)?, index)?;
                    Ok(result.len().to_string())
                }
            } else {
                // For other expressions, evaluate and get string length
                let result = evaluate_expr(inner, env, opt)?;
                Ok(result.len().to_string())
            }
        }

        ParamExpr::Defaulting { inner, colon, op, word } => {
            let inner_result = evaluate_expr(inner, env, opt);

            let should_use_default = match inner_result {
                Ok(ref value) => {
                    if *colon {
                        value.is_empty()
                    } else {
                        // For non-colon variants, check if variable is unset
                        match inner.as_ref() {
                            ParamExpr::Ref { target, .. } => !is_target_set(target, env),
                            _ => false,
                        }
                    }
                }
                Err(_) => true, // Variable is unset
            };

            handle_defaulting_operation(inner_result, should_use_default, op, word, env, opt)
        }

        ParamExpr::Remove { inner, op, pattern } => {
            let value = evaluate_expr(inner, env, opt)?;
            let pattern_str = evaluate_word_list(pattern, env, opt)?;

            match op {
                RemoveOp::Prefix { long } => remove_prefix(&value, &pattern_str, *long),
                RemoveOp::Suffix { long } => remove_suffix(&value, &pattern_str, *long),
            }
        }

        ParamExpr::Replace { inner, scope, pat, repl } => {
            let value = evaluate_expr(inner, env, opt)?;
            let pattern_str = evaluate_word_list(pat, env, opt)?;
            let replacement = evaluate_word_list(repl, env, opt)?;

            replace_pattern(&value, &pattern_str, &replacement, scope)
        }

        ParamExpr::Substring { inner, offset, len } => {
            let value = evaluate_expr(inner, env, opt)?;
            substring(&value, *offset, *len)
        }

        ParamExpr::Indirection { inner, style: _ } => {
            // First evaluate inner to get variable name
            let var_name = evaluate_expr(inner, env, opt)?;

            // Then look up that variable
            if let Some(value) = env.get(&var_name) {
                Ok(value.to_scalar())
            } else {
                Ok(String::new())
            }
        }

        ParamExpr::ZshFlags { flags, inner } => {
            // Special handling for join flag - it needs access to the original array
            if let Some(join_flag) = flags.iter().find(|f| matches!(f.kind, ZFlag::J { .. })) {
                if let ZFlag::J { sep } = &join_flag.kind {
                    if let ParamExpr::Ref {
                        target,
                        index: Index::None,
                    } = inner.as_ref()
                    {
                        let raw_value = get_target_value(target, env)?;
                        match &raw_value {
                            Value::Array(arr) => {
                                let mut result = arr.join(sep);
                                // Apply remaining flags to the joined result
                                for flag in flags {
                                    if !matches!(flag.kind, ZFlag::J { .. }) {
                                        result = apply_flag(&result, flag, env, opt)?;
                                    }
                                }
                                return Ok(result);
                            }
                            Value::Assoc(map) => {
                                let joined = map.values().map(|s| s.as_str()).collect::<Vec<_>>().join(sep);
                                let mut result = joined;
                                // Apply remaining flags to the joined result
                                for flag in flags {
                                    if !matches!(flag.kind, ZFlag::J { .. }) {
                                        result = apply_flag(&result, flag, env, opt)?;
                                    }
                                }
                                return Ok(result);
                            }
                            _ => {} // Fall through to normal processing
                        }
                    }
                }
            }

            // Normal flag processing
            let mut value = evaluate_expr(inner, env, opt)?;
            for flag in flags {
                value = apply_flag(&value, flag, env, opt)?;
            }
            Ok(value)
        }

        ParamExpr::Modifiers { inner, mods } => {
            let mut value = evaluate_expr(inner, env, opt)?;

            for modifier in mods {
                value = apply_path_modifier(&value, modifier);
            }

            Ok(value)
        }
    }
}

/// Handle defaulting operations (:-,  :=,  :+,  :?)
fn handle_defaulting_operation(
    inner_result: Result<String, Error>,
    should_use_default: bool,
    op: &DefaultOp,
    word: &[Word],
    env: &Env,
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
                // TODO: Implement assignment to env
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

/// Get value for a target (variable name, special parameter, or positional)
fn get_target_value(target: &Target, env: &Env) -> Result<Value, Error> {
    if let Some(special) = target.special {
        if let Some(val) = env.get_special(special) {
            Ok(Value::scalar(val))
        } else {
            Err(Error::Eval(format!("undefined special parameter: ${}", special)))
        }
    } else if let Some(pos) = target.positional {
        if let Some(val) = env.get_positional(pos) {
            Ok(Value::scalar(val))
        } else {
            Ok(Value::scalar(String::new()))
        }
    } else if let Some(value) = env.get(&target.name) {
        Ok(value.clone())
    } else {
        Err(Error::Eval(format!("undefined variable: {}", target.name)))
    }
}

/// Check if target is set (exists in environment)
fn is_target_set(target: &Target, env: &Env) -> bool {
    target.special.is_some() || target.positional.is_some() || env.is_set(&target.name)
}

/// Apply array/string indexing
fn apply_index(value: &Value, index: &Index) -> Result<String, Error> {
    match (value, index) {
        (_, Index::None) => Ok(value.to_scalar()),

        (Value::Array(arr), Index::One(i)) => {
            let idx = calculate_array_index(*i, arr.len())?;
            Ok(arr.get(idx).cloned().unwrap_or_default())
        }

        (Value::Assoc(map), Index::One(i)) => {
            let key = i.to_string();
            Ok(map.get(&key).cloned().unwrap_or_default())
        }

        (Value::Scalar(s), Index::One(i)) => {
            let chars: Vec<char> = s.chars().collect();
            let idx = calculate_array_index(*i, chars.len())?;
            Ok(chars.get(idx).map(|c| c.to_string()).unwrap_or_default())
        }

        (Value::Assoc(map), Index::Key(key)) => Ok(map.get(key).cloned().unwrap_or_default()),

        (_, Index::Key(_)) => Ok(String::new()), // For non-associative arrays

        (Value::Array(arr), Index::Slice(start, end)) => {
            let (start_idx, end_idx) = calculate_slice_indices(*start, *end, arr.len());
            let slice = arr.get(start_idx..end_idx).unwrap_or(&[]);
            Ok(slice.join(" "))
        }

        (Value::Scalar(s), Index::Slice(start, end)) => {
            let chars: Vec<char> = s.chars().collect();
            let (start_idx, end_idx) = calculate_slice_indices(*start, *end, chars.len());
            let slice = chars.get(start_idx..end_idx).unwrap_or(&[]);
            Ok(slice.iter().collect())
        }

        (Value::Assoc(_), Index::Slice(_, _)) => Ok(String::new()), // Unsupported

        (_, Index::StrSlice(offset, len)) => substring(&value.to_scalar(), *offset, Some(*len)),
    }
}

/// Calculate array index, handling negative indices
fn calculate_array_index(i: i64, len: usize) -> Result<usize, Error> {
    let idx = if i < 0 {
        let pos = len as i64 + i;
        if pos < 0 {
            return Ok(usize::MAX);
        } // Out of bounds
        pos as usize
    } else {
        if i == 0 {
            return Ok(usize::MAX);
        } // 1-based indexing, 0 is invalid
        (i - 1) as usize // Convert from 1-based to 0-based
    };
    Ok(idx)
}

/// Calculate slice indices for arrays and strings
fn calculate_slice_indices(start: i64, end: i64, len: usize) -> (usize, usize) {
    let start_idx = if start < 0 {
        0
    } else {
        ((start - 1) as usize).min(len) // Convert from 1-based to 0-based
    };

    let end_idx = if end < 0 {
        len
    } else {
        ((end - 1) as usize).min(len) // Convert from 1-based to 0-based
    };

    (start_idx, end_idx)
}

/// Evaluate a list of words
fn evaluate_word_list(words: &[Word], env: &Env, opt: &Options) -> Result<String, Error> {
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
                    // TODO: Execute command
                    return Err(Error::Unsupported("command substitution not implemented".to_string()));
                } else {
                    result.push_str(&format!("$({}", cmd));
                    result.push(')');
                }
            }
            Word::ArithSubst(expr) => {
                // TODO: Implement arithmetic evaluation
                result.push_str(&format!("$(({})", expr));
                result.push_str("))");
            }
        }
    }

    Ok(result)
}

/// Apply a zsh flag to a value
fn apply_flag(value: &str, flag: &ZshFlag, env: &Env, opt: &Options) -> Result<String, Error> {
    match &flag.kind {
        ZFlag::U => Ok(value.to_uppercase()),
        ZFlag::L => Ok(value.to_lowercase()),
        ZFlag::C => {
            if let Some(first_char) = value.chars().next() {
                let mut result = String::new();
                result.push(first_char.to_uppercase().next().unwrap_or(first_char));
                result.push_str(&value[first_char.len_utf8()..]);
                Ok(result)
            } else {
                Ok(value.to_string())
            }
        }
        ZFlag::Q => {
            // Shell quote - simple implementation
            if value.chars().any(|c| c.is_whitespace()) {
                Ok(format!("'{}'", value.replace('\'', "'\\''")))
            } else {
                Ok(value.to_string())
            }
        }
        ZFlag::Unquote => {
            // Remove shell quotes - simple implementation
            let len = value.len();
            if len >= 2 {
                let first = value.chars().next().unwrap();
                let last = value.chars().last().unwrap();

                if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
                    Ok(value.chars().skip(1).take(len - 2).collect())
                } else {
                    Ok(value.to_string())
                }
            } else {
                Ok(value.to_string())
            }
        }
        ZFlag::F => {
            // Split on newlines and rejoin with spaces
            Ok(value.lines().collect::<Vec<_>>().join(" "))
        }
        ZFlag::Z | ZFlag::ZExt => {
            // Shell word splitting - simple implementation
            Ok(value.split_whitespace().collect::<Vec<_>>().join(" "))
        }
        ZFlag::Unique => {
            let mut words: Vec<&str> = value.split_whitespace().collect();
            words.dedup();
            Ok(words.join(" "))
        }
        ZFlag::O => {
            let mut words: Vec<&str> = value.split_whitespace().collect();
            words.sort();
            Ok(words.join(" "))
        }
        ZFlag::ODesc => {
            let mut words: Vec<&str> = value.split_whitespace().collect();
            words.sort();
            words.reverse();
            Ok(words.join(" "))
        }
        ZFlag::K | ZFlag::V | ZFlag::T => {
            // These flags need access to the original variable
            // For now, return as-is
            Ok(value.to_string())
        }
        ZFlag::L2 { width, fill: _fill, pad } => {
            let w: usize = width.parse().unwrap_or(0);
            let pad_char = pad.chars().next().unwrap_or(' ');
            let formatted = format!("{:width$}", value, width = w);
            if pad_char == ' ' {
                Ok(formatted)
            } else {
                Ok(formatted.replace(' ', &pad_char.to_string()))
            }
        }
        ZFlag::R2 { width, fill: _fill, pad } => {
            let w: usize = width.parse().unwrap_or(0);
            let pad_char = pad.chars().next().unwrap_or(' ');
            let formatted = format!("{:>width$}", value, width = w);
            if pad_char == ' ' {
                Ok(formatted)
            } else {
                Ok(formatted.replace(' ', &pad_char.to_string()))
            }
        }
        ZFlag::J { sep: _ } => {
            // Join array elements - but we can't access the original array here
            // This needs to be handled at a higher level
            Ok(value.to_string())
        }
        ZFlag::S { sep } => {
            // Split string on separator
            Ok(value.split(sep).collect::<Vec<_>>().join(" "))
        }
        ZFlag::VDisplay => {
            // Display with escape sequences visible
            Ok(value
                .chars()
                .map(|c| match c {
                    '\n' => "\\n".to_string(),
                    '\t' => "\\t".to_string(),
                    '\r' => "\\r".to_string(),
                    '\\' => "\\\\".to_string(),
                    _ => c.to_string(),
                })
                .collect())
        }
        ZFlag::P => {
            // Indirection - look up variable by name
            if let Some(val) = env.get(value) {
                Ok(val.to_scalar())
            } else {
                Ok(String::new())
            }
        }
        ZFlag::E => {
            if opt.allow_flag_e {
                // Re-expand the result
                expand_str(value, env, opt)
            } else {
                Err(Error::Unsupported("(e) flag disabled for security".to_string()))
            }
        }
    }
}

/// Apply path modifier to a value
fn apply_path_modifier(value: &str, modifier: &PathMod) -> String {
    let path = Path::new(value);

    match modifier {
        PathMod::H => {
            // dirname
            path.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        }
        PathMod::T => {
            // basename
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| value.to_string())
        }
        PathMod::R => {
            // root (remove extension)
            path.with_extension("").to_string_lossy().to_string()
        }
        PathMod::E => {
            // extension
            path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
        }
        PathMod::A | PathMod::LowerA => {
            // realpath - just return as-is for now (TODO: implement proper realpath)
            value.to_string()
        }
    }
}

/// Remove prefix matching pattern
fn remove_prefix(value: &str, pattern: &str, long: bool) -> Result<String, Error> {
    if long {
        // Longest match - find the longest prefix that matches pattern
        let mut best_match = 0;
        for i in 1..=value.len() {
            let prefix = &value[..i];
            if glob_match(pattern, prefix) {
                best_match = i;
            }
        }
        Ok(value[best_match..].to_string())
    } else {
        // Shortest match - find first matching prefix
        for i in 1..=value.len() {
            let prefix = &value[..i];
            if glob_match(pattern, prefix) {
                return Ok(value[i..].to_string());
            }
        }
        Ok(value.to_string())
    }
}

/// Remove suffix matching pattern
fn remove_suffix(value: &str, pattern: &str, long: bool) -> Result<String, Error> {
    if long {
        // Longest match
        let mut best_match = value.len();
        for i in 0..value.len() {
            let suffix = &value[i..];
            if glob_match(pattern, suffix) {
                best_match = i;
            }
        }
        Ok(value[..best_match].to_string())
    } else {
        // Shortest match
        for i in (0..value.len()).rev() {
            let suffix = &value[i..];
            if glob_match(pattern, suffix) {
                return Ok(value[..i].to_string());
            }
        }
        Ok(value.to_string())
    }
}

/// Replace pattern in value
fn replace_pattern(value: &str, pattern: &str, replacement: &str, scope: &ReplaceScope) -> Result<String, Error> {
    match scope {
        ReplaceScope::First => {
            // Replace first occurrence
            if let Some(pos) = find_pattern_match(value, pattern) {
                let mut result = String::new();
                result.push_str(&value[..pos.0]);
                result.push_str(replacement);
                result.push_str(&value[pos.1..]);
                Ok(result)
            } else {
                Ok(value.to_string())
            }
        }
        ReplaceScope::Global => {
            // Replace all occurrences
            let mut result = value.to_string();
            let mut matches = find_all_pattern_matches(value, pattern);
            matches.reverse(); // Process from end to start to maintain indices

            for (start, end) in matches {
                result.replace_range(start..end, replacement);
            }

            Ok(result)
        }
        ReplaceScope::AnchorPrefix => {
            // Replace if pattern matches at start
            if let Some(stripped) = value.strip_prefix(pattern) {
                Ok(format!("{}{}", replacement, stripped))
            } else {
                Ok(value.to_string())
            }
        }
        ReplaceScope::AnchorSuffix => {
            // Replace if pattern matches at end
            if let Some(stripped) = value.strip_suffix(pattern) {
                Ok(format!("{}{}", stripped, replacement))
            } else {
                Ok(value.to_string())
            }
        }
    }
}

/// Extract substring
fn substring(value: &str, offset: i64, len: Option<i64>) -> Result<String, Error> {
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

/// Simple glob matching implementation
pub fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_recursive(pattern, text)
}

fn glob_match_recursive(pattern: &str, text: &str) -> bool {
    let pat_chars: Vec<char> = pattern.chars().collect();
    let txt_chars: Vec<char> = text.chars().collect();

    glob_match_chars(&pat_chars, &txt_chars, 0, 0)
}

fn glob_match_chars(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
    if pi >= pattern.len() {
        return ti >= text.len();
    }

    match pattern[pi] {
        '*' => {
            // Try matching zero or more characters
            if glob_match_chars(pattern, text, pi + 1, ti) {
                return true;
            }
            if ti < text.len() {
                glob_match_chars(pattern, text, pi, ti + 1)
            } else {
                false
            }
        }
        '?' => {
            if ti < text.len() {
                glob_match_chars(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
        '[' => {
            // Character class - simplified implementation
            if ti >= text.len() {
                return false;
            }

            let mut end_bracket = pi + 1;
            while end_bracket < pattern.len() && pattern[end_bracket] != ']' {
                end_bracket += 1;
            }

            if end_bracket >= pattern.len() {
                // No closing bracket, treat as literal
                if pattern[pi] == text[ti] {
                    glob_match_chars(pattern, text, pi + 1, ti + 1)
                } else {
                    false
                }
            } else {
                let class = &pattern[pi + 1..end_bracket];
                let ch = text[ti];

                if class.contains(&ch) {
                    glob_match_chars(pattern, text, end_bracket + 1, ti + 1)
                } else {
                    false
                }
            }
        }
        _c => {
            if ti < text.len() && pattern[pi] == text[ti] {
                glob_match_chars(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}

/// Find first pattern match position
fn find_pattern_match(text: &str, pattern: &str) -> Option<(usize, usize)> {
    text.find(pattern).map(|pos| (pos, pos + pattern.len()))
}

/// Find all pattern match positions  
fn find_all_pattern_matches(text: &str, pattern: &str) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    let mut start = 0;

    while let Some(pos) = text[start..].find(pattern) {
        let abs_pos = start + pos;
        matches.push((abs_pos, abs_pos + pattern.len()));
        start = abs_pos + pattern.len();
    }

    matches
}
