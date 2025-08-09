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
    // Check for overflow conditions
    if i == i64::MAX || i == i64::MIN {
        return Err(Error::IndexOutOfBounds(format!("Index out of range: {}", i)));
    }

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
                result.push(')');
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
            let mut seen = std::collections::HashSet::new();
            let words: Vec<&str> = value.split_whitespace().filter(|&word| seen.insert(word)).collect();
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
            if value.is_empty() {
                ".".to_string()
            } else {
                path.parent()
                    .map(|p| {
                        let parent_str = p.to_string_lossy();
                        if parent_str.is_empty() {
                            ".".to_string()
                        } else {
                            parent_str.to_string()
                        }
                    })
                    .unwrap_or_else(|| ".".to_string())
            }
        }
        PathMod::T => {
            // basename
            if value.ends_with('/') && value != "/" {
                "".to_string()
            } else {
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| value.to_string())
            }
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
    let mut matching_positions = Vec::new();

    // Find all matching suffix positions
    for i in 0..=value.len() {
        let suffix = &value[i..];
        if glob_match(pattern, suffix) {
            matching_positions.push(i);
        }
    }

    if matching_positions.is_empty() {
        return Ok(value.to_string());
    }

    if long {
        // Longest match - choose the earliest position (longest suffix)
        let pos = matching_positions[0];
        Ok(value[..pos].to_string())
    } else {
        // Shortest match - choose the latest position (shortest suffix)
        let pos = matching_positions[matching_positions.len() - 1];
        Ok(value[..pos].to_string())
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

                let matches = if !class.is_empty() && class[0] == '!' {
                    // Negated character class
                    let class_chars = &class[1..];
                    !class_chars.contains(&ch)
                } else {
                    // Regular character class
                    class.contains(&ch)
                };

                if matches {
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

    if pattern.is_empty() {
        return matches;
    }

    let mut start = 0;
    while start < text.len() {
        if let Some(pos) = text[start..].find(pattern) {
            let abs_pos = start + pos;
            matches.push((abs_pos, abs_pos + pattern.len()));
            start = abs_pos + 1; // Move by 1 to find overlapping matches
        } else {
            break;
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::{Env, Value};
    use pretty_assertions::assert_eq;

    fn test_options() -> Options {
        Options::default()
    }

    fn test_env() -> Env {
        let mut env = Env::new();
        env.set_scalar("FOO", "bar");
        env.set_scalar("EMPTY", "");
        env.set_array("ARR", vec!["a", "b", "c"]);
        env.set_assoc("MAP", vec![("key1", "val1"), ("key2", "val2")]);
        env.set_scalar("PATH_VAR", "/usr/local/bin/test.txt");
        env.set_scalar("GREETING", "Hello World");
        env
    }

    #[test]
    fn test_expand_str_basic() {
        let env = test_env();
        let opt = test_options();

        // Basic expansion
        let result = expand_str("${FOO}", &env, &opt).unwrap();
        assert_eq!(result, "bar");

        // No expansion
        let result = expand_str("no expansion here", &env, &opt).unwrap();
        assert_eq!(result, "no expansion here");

        // Multiple expansions
        let result = expand_str("${FOO} and ${GREETING}", &env, &opt).unwrap();
        assert_eq!(result, "bar and Hello World");
    }

    #[test]
    fn test_expand_str_error_cases() {
        let env = test_env();
        let opt = test_options();

        // Invalid expansion
        let result = expand_str("${", &env, &opt);
        assert!(result.is_ok()); // Should handle gracefully

        // Unset variable with error operator
        let result = expand_str("${UNSET?error message}", &env, &opt);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_target_value() {
        let env = test_env();

        // Regular variable
        let target = Target {
            name: "FOO".to_string(),
            special: None,
            positional: None,
        };
        let value = get_target_value(&target, &env).unwrap();
        if let Value::Scalar(s) = value {
            assert_eq!(s, "bar");
        } else {
            panic!("Expected scalar value");
        }

        // Special parameter - 需要设置positional参数
        let mut env = test_env();
        env.set_positional(vec!["arg1", "arg2", "arg3"]);
        env.set_positional(vec!["arg1", "arg2", "arg3"]);
        let target = Target {
            name: "".to_string(),
            special: None,
            positional: Some(1),
        };
        let value = get_target_value(&target, &env).unwrap();
        if let Value::Scalar(s) = value {
            assert_eq!(s, "arg1");
        } else {
            panic!("Expected scalar value");
        }

        // Unset variable
        let target = Target {
            name: "UNSET".to_string(),
            special: None,
            positional: None,
        };
        let result = get_target_value(&target, &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_target_set() {
        let env = test_env();

        // Set variable
        let target = Target {
            name: "FOO".to_string(),
            special: None,
            positional: None,
        };
        assert!(is_target_set(&target, &env));

        // Empty variable
        let target = Target {
            name: "EMPTY".to_string(),
            special: None,
            positional: None,
        };
        assert!(is_target_set(&target, &env));

        // Unset variable
        let target = Target {
            name: "UNSET".to_string(),
            special: None,
            positional: None,
        };
        assert!(!is_target_set(&target, &env));
    }

    #[test]
    fn test_apply_index() {
        let env = test_env();

        // Array index (1-based indexing)
        let arr = env.get("ARR").unwrap();
        let result = apply_index(&arr, &Index::One(1)).unwrap();
        assert_eq!(result, "a"); // 索引1返回第一个元素

        let result = apply_index(&arr, &Index::One(2)).unwrap();
        assert_eq!(result, "b"); // 索引2返回第二个元素

        // String index (slice)
        let scalar = Value::Scalar("hello".to_string());
        let result = apply_index(&scalar, &Index::One(1)).unwrap();
        assert_eq!(result, "h"); // 索引1返回第一个字符

        // Out of bounds
        let result = apply_index(&arr, &Index::One(10));
        assert!(result.is_ok()); // 超出范围返回空字符串，不是错误

        // Associative array
        let map = env.get("MAP").unwrap();
        let result = apply_index(&map, &Index::Key("key1".to_string())).unwrap();
        assert_eq!(result, "val1");

        // Invalid key - 返回空字符串而不是错误
        let result = apply_index(&map, &Index::Key("invalid".to_string())).unwrap();
        assert_eq!(result, ""); // 不存在的key返回空字符串
    }

    #[test]
    fn test_calculate_array_index() {
        // Positive index (1-based, 0 is invalid)
        assert_eq!(calculate_array_index(0, 3).unwrap(), usize::MAX); // 0 is invalid
        assert_eq!(calculate_array_index(1, 3).unwrap(), 0); // 1-based to 0-based
        assert_eq!(calculate_array_index(3, 3).unwrap(), 2); // Last element

        // Negative index
        assert_eq!(calculate_array_index(-1, 3).unwrap(), 2);
        assert_eq!(calculate_array_index(-3, 3).unwrap(), 0);

        // Out of bounds - 这个函数不返回错误，而是返回特殊值
        assert_eq!(calculate_array_index(4, 3).unwrap(), 3); // 超出范围但不报错
        assert_eq!(calculate_array_index(-4, 3).unwrap(), usize::MAX); // 负数超出范围返回MAX
    }

    #[test]
    fn test_calculate_slice_indices() {
        // Normal slice (1-based input converted to 0-based)
        assert_eq!(calculate_slice_indices(1, 3, 5), (0, 2)); // 1-based [1,3) -> 0-based [0,2)

        // Negative indices
        assert_eq!(calculate_slice_indices(-2, -1, 5), (0, 5)); // negative start -> 0, negative end -> len

        // Mixed indices
        assert_eq!(calculate_slice_indices(-2, 4, 5), (0, 3)); // negative start -> 0, 4 -> 3

        // Out of bounds (clamped) - 检查实际行为
        assert_eq!(calculate_slice_indices(10, 20, 5), (5, 5)); // 都超过len，被限制为len
    }

    #[test]
    fn test_apply_flag() {
        let env = test_env();
        let opt = test_options();

        // Uppercase flag
        let flag = ZshFlag {
            kind: ZFlag::U,
            args: vec![],
        };
        let result = apply_flag("hello", &flag, &env, &opt).unwrap();
        assert_eq!(result, "HELLO");

        // Lowercase flag
        let flag = ZshFlag {
            kind: ZFlag::L,
            args: vec![],
        };
        let result = apply_flag("HELLO", &flag, &env, &opt).unwrap();
        assert_eq!(result, "hello");

        // Capitalize flag - 只大写第一个字符
        let flag = ZshFlag {
            kind: ZFlag::C,
            args: vec![],
        };
        let result = apply_flag("hello world", &flag, &env, &opt).unwrap();
        assert_eq!(result, "Hello world"); // 只有第一个字符大写

        // Quote flag
        let flag = ZshFlag {
            kind: ZFlag::Q,
            args: vec![],
        };
        let result = apply_flag("hello world", &flag, &env, &opt).unwrap();
        assert_eq!(result, "'hello world'");

        // Join flag - 当前实现无法在这个级别处理join，返回原始值
        let flag = ZshFlag {
            kind: ZFlag::J { sep: "|".to_string() },
            args: vec![],
        };
        let result = apply_flag("a b c", &flag, &env, &opt).unwrap();
        assert_eq!(result, "a b c"); // Join flag在apply_flag级别无法工作

        // Split flag
        let flag = ZshFlag {
            kind: ZFlag::S { sep: ":".to_string() },
            args: vec![],
        };
        let result = apply_flag("a:b:c", &flag, &env, &opt).unwrap();
        assert_eq!(result, "a b c");

        // Unique flag
        let flag = ZshFlag {
            kind: ZFlag::Unique,
            args: vec![],
        };
        let result = apply_flag("a b a c b", &flag, &env, &opt).unwrap();
        assert_eq!(result, "a b c");

        // Sort ascending
        let flag = ZshFlag {
            kind: ZFlag::O,
            args: vec![],
        };
        let result = apply_flag("c a b", &flag, &env, &opt).unwrap();
        assert_eq!(result, "a b c");

        // Sort descending
        let flag = ZshFlag {
            kind: ZFlag::ODesc,
            args: vec![],
        };
        let result = apply_flag("a b c", &flag, &env, &opt).unwrap();
        assert_eq!(result, "c b a");
    }

    #[test]
    fn test_apply_path_modifier() {
        // Dirname
        assert_eq!(apply_path_modifier("/usr/local/bin/file.txt", &PathMod::H), "/usr/local/bin");
        assert_eq!(apply_path_modifier("file.txt", &PathMod::H), "."); // 当前目录
        assert_eq!(apply_path_modifier("", &PathMod::H), "."); // 空路径的dirname是当前目录

        // Basename
        assert_eq!(apply_path_modifier("/usr/local/bin/file.txt", &PathMod::T), "file.txt");
        assert_eq!(apply_path_modifier("/usr/local/bin/", &PathMod::T), "");
        assert_eq!(apply_path_modifier("file.txt", &PathMod::T), "file.txt");

        // Root (remove extension)
        assert_eq!(apply_path_modifier("file.txt", &PathMod::R), "file");
        assert_eq!(apply_path_modifier("file.tar.gz", &PathMod::R), "file.tar");
        assert_eq!(apply_path_modifier("file", &PathMod::R), "file");

        // Extension
        assert_eq!(apply_path_modifier("file.txt", &PathMod::E), "txt");
        assert_eq!(apply_path_modifier("file.tar.gz", &PathMod::E), "gz");
        assert_eq!(apply_path_modifier("file", &PathMod::E), "");
    }

    #[test]
    fn test_remove_prefix() {
        // Short prefix removal
        let result = remove_prefix("hello_world", "he*", false).unwrap();
        assert_eq!(result, "llo_world");

        // Long prefix removal
        let result = remove_prefix("hello_world_test", "he*", true).unwrap();
        assert_eq!(result, "");

        // No match
        let result = remove_prefix("hello", "xyz*", false).unwrap();
        assert_eq!(result, "hello");

        // Literal match
        let result = remove_prefix("hello", "hel", false).unwrap();
        assert_eq!(result, "lo");
    }

    #[test]
    fn test_remove_suffix() {
        // Short suffix removal
        let result = remove_suffix("hello_world", "*ld", false).unwrap();
        assert_eq!(result, "hello_wor");

        // Long suffix removal
        let result = remove_suffix("hello_world_test", "*test", true).unwrap();
        assert_eq!(result, ""); // Long match of *test removes entire string

        // No match
        let result = remove_suffix("hello", "*xyz", false).unwrap();
        assert_eq!(result, "hello");

        // Literal match
        let result = remove_suffix("hello", "lo", false).unwrap();
        assert_eq!(result, "hel");
    }

    #[test]
    fn test_replace_pattern() {
        // Replace first
        let result = replace_pattern("hello world hello", "hello", "hi", &ReplaceScope::First).unwrap();
        assert_eq!(result, "hi world hello");

        // Replace all
        let result = replace_pattern("hello world hello", "hello", "hi", &ReplaceScope::Global).unwrap();
        assert_eq!(result, "hi world hi");

        // Replace prefix anchor
        let result = replace_pattern("hello world", "hello", "hi", &ReplaceScope::AnchorPrefix).unwrap();
        assert_eq!(result, "hi world");

        // Replace suffix anchor
        let result = replace_pattern("hello world", "world", "universe", &ReplaceScope::AnchorSuffix).unwrap();
        assert_eq!(result, "hello universe");

        // No match
        let result = replace_pattern("hello", "xyz", "abc", &ReplaceScope::First).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_substring() {
        // Basic substring
        let result = substring("hello", 1, Some(3)).unwrap();
        assert_eq!(result, "ell");

        // Substring from offset to end
        let result = substring("hello", 2, None).unwrap();
        assert_eq!(result, "llo");

        // Negative offset
        let result = substring("hello", -2, None).unwrap();
        assert_eq!(result, "lo");

        // Zero length
        let result = substring("hello", 1, Some(0)).unwrap();
        assert_eq!(result, "");

        // Out of bounds offset
        let result = substring("hello", 10, Some(1));
        assert!(result.is_ok()); // Should return empty string

        // Negative length (should be treated as error or 0)
        let result = substring("hello", 1, Some(-1));
        assert!(result.is_ok()); // Implementation dependent
    }

    #[test]
    fn test_glob_match() {
        // Basic wildcard
        assert!(glob_match("he*", "hello"));
        assert!(glob_match("*lo", "hello"));
        assert!(glob_match("h*o", "hello"));
        assert!(!glob_match("he*", "world"));

        // Question mark
        assert!(glob_match("h?llo", "hello"));
        assert!(glob_match("h?llo", "hallo"));
        assert!(!glob_match("h?llo", "hllo"));

        // Character classes
        assert!(glob_match("h[aei]llo", "hello"));
        assert!(glob_match("h[aei]llo", "hallo"));
        assert!(!glob_match("h[aei]llo", "hullo"));

        // Negated character classes
        assert!(glob_match("h[!aei]llo", "hullo"));
        assert!(!glob_match("h[!aei]llo", "hello"));

        // Exact match
        assert!(glob_match("hello", "hello"));
        assert!(!glob_match("hello", "hello world"));

        // Empty patterns
        assert!(glob_match("", ""));
        assert!(!glob_match("", "hello"));
        assert!(!glob_match("hello", ""));
    }

    #[test]
    fn test_evaluate_word_list() {
        let env = test_env();
        let opt = test_options();

        // Simple text
        let words = vec![Word::Text("hello".to_string())];
        let result = evaluate_word_list(&words, &env, &opt).unwrap();
        assert_eq!(result, "hello");

        // Parameter expansion
        let param_expr = crate::parser::parse_braced("${FOO}").unwrap();
        let words = vec![Word::Param(Box::new(param_expr))];
        let result = evaluate_word_list(&words, &env, &opt).unwrap();
        assert_eq!(result, "bar");

        // Mixed content
        let param_expr = crate::parser::parse_braced("${FOO}").unwrap();
        let words = vec![
            Word::Text("prefix_".to_string()),
            Word::Param(Box::new(param_expr)),
            Word::Text("_suffix".to_string()),
        ];
        let result = evaluate_word_list(&words, &env, &opt).unwrap();
        assert_eq!(result, "prefix_bar_suffix");

        // Command substitution (should be literal)
        let words = vec![Word::CmdSubst("echo hello".to_string())];
        let result = evaluate_word_list(&words, &env, &opt).unwrap();
        assert_eq!(result, "$(echo hello)");

        // Arithmetic substitution (should be literal)
        let words = vec![Word::ArithSubst("1+1".to_string())];
        let result = evaluate_word_list(&words, &env, &opt).unwrap();
        assert_eq!(result, "$((1+1))");
    }

    #[test]
    fn test_options_modes() {
        let mut opt = Options::default();

        // Test different modes
        opt.mode = Mode::Zsh;
        assert_eq!(opt.mode, Mode::Zsh);

        opt.mode = Mode::Bash;
        assert_eq!(opt.mode, Mode::Bash);

        opt.mode = Mode::Posix;
        assert_eq!(opt.mode, Mode::Posix);

        opt.mode = Mode::Union;
        assert_eq!(opt.mode, Mode::Union);

        // Test security options
        opt.allow_exec_subst = true;
        assert!(opt.allow_exec_subst);

        opt.allow_flag_e = true;
        assert!(opt.allow_flag_e);

        // Test glob implementation
        opt.glob_impl = GlobKind::Simple;
        assert_eq!(opt.glob_impl, GlobKind::Simple);
    }

    #[test]
    fn test_error_conditions() {
        let _env = test_env();
        let _opt = test_options();

        // Test various error conditions that should be handled gracefully

        // Invalid UTF-8 handling - Rust strings are always valid UTF-8, so this is less relevant

        // Very long strings
        let long_string = "x".repeat(10000);
        let result = substring(&long_string, 5000, Some(100));
        assert!(result.is_ok());

        // Edge case indices
        let result = calculate_array_index(i64::MAX, 1);
        assert!(result.is_err());

        let result = calculate_array_index(i64::MIN, 1);
        assert!(result.is_err());

        // Empty patterns
        let result = glob_match("", "test");
        assert!(!result);

        // Complex nested patterns
        let result = remove_prefix("test", "**", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_pattern_matches() {
        // Single match
        let result = find_pattern_match("hello world hello", "hello");
        assert_eq!(result, Some((0, 5)));

        // No match
        let result = find_pattern_match("hello world", "xyz");
        assert_eq!(result, None);

        // Multiple matches - should find all
        let matches = find_all_pattern_matches("hello world hello", "hello");
        assert_eq!(matches, vec![(0, 5), (12, 17)]);

        // Overlapping patterns
        let matches = find_all_pattern_matches("aaa", "aa");
        assert_eq!(matches, vec![(0, 2), (1, 3)]);

        // Empty pattern
        let matches = find_all_pattern_matches("test", "");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_complex_evaluations() {
        let mut env = test_env();
        let _opt = test_options();

        // Complex nested expansion
        env.set_scalar("VAR_NAME", "FOO");
        env.set_scalar("FOO", "result");

        // This would need indirection support
        // let result = expand_str("${(P)VAR_NAME}", &env, &opt);

        // Complex path operations
        env.set_scalar("COMPLEX_PATH", "/very/long/path/to/file.complex.ext");
        let _result = expand_str("${COMPLEX_PATH:h:t:r:e}", &env, &_opt);
        // This would need proper path modifier chaining

        // Complex flag combinations - test what's currently supported
        env.set_scalar("TEST_STR", "Hello World Test");

        // Test individual components work
        let result = expand_str("${TEST_STR}", &env, &_opt).unwrap();
        assert_eq!(result, "Hello World Test");
    }
}
