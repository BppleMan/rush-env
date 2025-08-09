use super::Options;
use crate::ast::*;
use crate::env::{EnvVars, Value};

/// Main expansion function - expands all parameter expansions in input string
pub fn expand_str<E: EnvVars + ?Sized>(input: &str, env: &E, opt: &Options) -> Result<String, Error> {
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
pub fn evaluate_expr<E: EnvVars + ?Sized>(expr: &ParamExpr, env: &E, opt: &Options) -> Result<String, Error> {
    match expr {
        ParamExpr::Ref { target, index } => {
            let value = super::get_target_value(target, env)?;
            super::apply_index(&value, index)
        }

        ParamExpr::Length { inner } => {
            // For length operation with arrays, we need special handling
            // ${#ARR} should return the length of the first element, not the joined string
            if let ParamExpr::Ref { target, index } = inner.as_ref() {
                if let Index::None = index {
                    let value = super::get_target_value(target, env)?;
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
                    let result = super::apply_index(&super::get_target_value(target, env)?, index)?;
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
                            ParamExpr::Ref { target, .. } => !super::is_target_set(target, env),
                            _ => false,
                        }
                    }
                }
                Err(_) => true, // Variable is unset
            };

            super::handle_defaulting_operation(inner_result, should_use_default, op, word, env, opt)
        }

        ParamExpr::Remove { inner, op, pattern } => {
            let value = evaluate_expr(inner, env, opt)?;
            let pattern_str = super::evaluate_word_list(pattern, env, opt)?;

            match op {
                RemoveOp::Prefix { long } => super::remove_prefix(&value, &pattern_str, *long),
                RemoveOp::Suffix { long } => super::remove_suffix(&value, &pattern_str, *long),
            }
        }

        ParamExpr::Replace { inner, scope, pat, repl } => {
            let value = evaluate_expr(inner, env, opt)?;
            let pattern_str = super::evaluate_word_list(pat, env, opt)?;
            let replacement = super::evaluate_word_list(repl, env, opt)?;

            super::replace_pattern(&value, &pattern_str, &replacement, scope)
        }

        ParamExpr::Substring { inner, offset, len } => {
            let value = evaluate_expr(inner, env, opt)?;
            super::substring(&value, *offset, *len)
        }

        ParamExpr::Indirection { inner, style: _ } => {
            // First evaluate inner to get variable name
            let var_name = evaluate_expr(inner, env, opt)?;

            // Then look up that variable
            if let Some(value) = env.get_var(&var_name) {
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
                        let raw_value = super::get_target_value(target, env)?;
                        match &raw_value {
                            Value::Array(arr) => {
                                let mut result = arr.join(sep);
                                // Apply remaining flags to the joined result
                                for flag in flags {
                                    if !matches!(flag.kind, ZFlag::J { .. }) {
                                        result = super::apply_flag(&result, flag, env, opt)?;
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
                                        result = super::apply_flag(&result, flag, env, opt)?;
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
                value = super::apply_flag(&value, flag, env, opt)?;
            }
            Ok(value)
        }

        ParamExpr::Modifiers { inner, mods } => {
            let mut value = evaluate_expr(inner, env, opt)?;

            for modifier in mods {
                value = super::apply_path_modifier(&value, modifier);
            }

            Ok(value)
        }
    }
}
