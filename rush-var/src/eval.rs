use crate::ast::*;
use crate::env::Env;

#[derive(Debug, Clone)]
pub enum Mode {
    Zsh,
    Bash,
    Posix,
    Union,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub mode: Mode,
    pub allow_exec_subst: bool,
    pub allow_flag_e: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            mode: Mode::Zsh,
            allow_exec_subst: false,
            allow_flag_e: false,
        }
    }
}

pub fn expand_str(input: &str, env: &Env, opt: &Options) -> Result<String, Error> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some('{') = chars.peek().copied() {
                chars.next();
                let mut depth = 1;
                let mut expr = String::new();
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
                let parsed = crate::parser::parse_braced(&format!("${{{}}}", expr))?;
                let val = eval(&parsed, env, opt)?;
                result.push_str(&val);
            } else {
                let mut name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '_' || ch.is_ascii_alphanumeric() {
                        name.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    result.push('$');
                } else {
                    let val = env.get_scalar(&name).unwrap_or_default();
                    result.push_str(val);
                }
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

pub fn eval(expr: &ParamExpr, env: &Env, _opt: &Options) -> Result<String, Error> {
    match expr {
        ParamExpr::Ref { target } => Ok(env.get_scalar(&target.name).unwrap_or_default().to_string()),
        ParamExpr::Length { inner } => {
            let val = eval(inner, env, _opt)?;
            Ok(val.chars().count().to_string())
        }
        ParamExpr::Defaulting {
            inner,
            colon,
            op,
            word,
        } => {
            let raw = eval(inner, env, _opt)?;
            let is_empty = raw.is_empty();
            let apply = if *colon {
                is_empty
            } else {
                env.get_scalar(match inner.as_ref() {
                    ParamExpr::Ref { target } => target.name.as_str(),
                    _ => "",
                })
                .is_none()
            };
            match op {
                DefaultOp::Dash => {
                    if apply { Ok(word.clone()) } else { Ok(raw) }
                }
                DefaultOp::Assign => {
                    if apply { Ok(word.clone()) } else { Ok(raw) }
                }
                DefaultOp::Plus => {
                    if apply { Ok(String::new()) } else { Ok(word.clone()) }
                }
                DefaultOp::QMark => {
                    if apply {
                        Err(Error::Eval(word.clone()))
                    } else {
                        Ok(raw)
                    }
                }
            }
        }
        ParamExpr::Remove {
            inner,
            op,
            pattern,
        } => {
            let val = eval(inner, env, _opt)?;
            let res = match op {
                RemoveOp::Prefix { long: _ } => {
                    if val.starts_with(pattern) {
                        val[pattern.len()..].to_string()
                    } else {
                        val
                    }
                }
                RemoveOp::Suffix { long: _ } => {
                    if val.ends_with(pattern) {
                        val[..val.len() - pattern.len()].to_string()
                    } else {
                        val
                    }
                }
            };
            Ok(res)
        }
        ParamExpr::Replace {
            inner,
            scope,
            pat,
            repl,
        } => {
            let val = eval(inner, env, _opt)?;
            let res = match scope {
                ReplaceScope::First => val.replacen(pat, repl, 1),
                ReplaceScope::Global => val.replace(pat, repl),
                ReplaceScope::AnchorPrefix => {
                    if val.starts_with(pat) {
                        format!("{}{}", repl, &val[pat.len()..])
                    } else {
                        val
                    }
                }
                ReplaceScope::AnchorSuffix => {
                    if val.ends_with(pat) {
                        format!("{}{}", &val[..val.len() - pat.len()], repl)
                    } else {
                        val
                    }
                }
            };
            Ok(res)
        }
        ParamExpr::Substring {
            inner,
            offset,
            len,
        } => {
            let val = eval(inner, env, _opt)?;
            let chars: Vec<char> = val.chars().collect();
            let total = chars.len() as i64;
            let mut off = *offset;
            if off < 0 {
                off = total + off;
            }
            let off = off.max(0).min(total) as usize;
            let end = if let Some(l) = len {
                let mut end = off as i64 + *l;
                if end < 0 {
                    end = 0;
                }
                end.min(total) as usize
            } else {
                chars.len()
            };
            Ok(chars[off..end].iter().collect())
        }
        ParamExpr::Indirection { inner } => {
            let name = eval(inner, env, _opt)?;
            Ok(env.get_scalar(&name).unwrap_or_default().to_string())
        }
    }
}
