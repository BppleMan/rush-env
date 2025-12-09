use crate::env_source::VarSrc;
use std::iter::Peekable;
use std::str::Chars;

pub(crate) const LITERAL_DOLLAR_SENTINEL: &str = "__RUSH_VAR_DOLLAR__";

#[derive(Debug, Clone, PartialEq, Eq)]
enum VarOp {
    /// Plain lookup: `${VAR}` or `$VAR`
    Raw,
    /// `${VAR:-fallback}` (fallback when unset or empty) or `${VAR-fallback}` (fallback only when unset)
    Default { fallback: String, treat_empty_as_unset: bool },
    /// `${VAR:+alt}` (alt when set and non-empty) or `${VAR+alt}` (alt when set even if empty)
    Alternate { alt: String, require_non_empty: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VarExpr {
    name: String,
    op: VarOp,
}

impl VarExpr {
    fn apply(&self, env: &impl VarSrc) -> String {
        let value = env.get(&self.name);
        match (&self.op, value) {
            (VarOp::Raw, Some(v)) => v,
            (VarOp::Raw, None) => String::new(),
            (
                VarOp::Default {
                    fallback,
                    treat_empty_as_unset,
                },
                Some(v),
            ) => {
                if *treat_empty_as_unset && v.is_empty() {
                    expand(fallback, env)
                } else {
                    v
                }
            }
            (VarOp::Default { fallback, .. }, None) => expand(fallback, env),
            (VarOp::Alternate { alt, require_non_empty }, Some(v)) => {
                if *require_non_empty && v.is_empty() {
                    String::new()
                } else {
                    expand(alt, env)
                }
            }
            (VarOp::Alternate { .. }, None) => String::new(),
        }
    }
}

pub(crate) fn expand(input: &str, env: &impl VarSrc) -> String {
    let mut output = String::new();
    let mut literal = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '$' {
            literal.push(ch);
            continue;
        }

        match chars.peek() {
            Some('$') => {
                chars.next();
                literal.push_str(LITERAL_DOLLAR_SENTINEL);
            }
            Some('{') => {
                chars.next(); // consume '{'
                flush_literal(&mut output, &mut literal);
                let expr = parse_braced_expr(&mut chars);
                output.push_str(&expr.apply(env));
            }
            Some(next) if is_valid_unbraced_start(*next) => {
                flush_literal(&mut output, &mut literal);
                let expr = parse_unbraced_expr(&mut chars);
                output.push_str(&expr.apply(env));
            }
            _ => literal.push('$'),
        }
    }

    output.push_str(&literal);
    output
}

fn flush_literal(output: &mut String, literal: &mut String) {
    if literal.is_empty() {
        return;
    }

    output.push_str(literal);
    literal.clear();
}

fn parse_unbraced_expr(chars: &mut Peekable<Chars<'_>>) -> VarExpr {
    let mut name = String::new();
    if let Some(ch) = chars.next() {
        name.push(ch);

        if is_name_start(ch) {
            while let Some(&next) = chars.peek() {
                if is_name_continue(next) {
                    name.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }

    VarExpr { name, op: VarOp::Raw }
}

fn parse_braced_expr(chars: &mut Peekable<Chars<'_>>) -> VarExpr {
    let mut name = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '}' => {
                chars.next();
                return VarExpr { name, op: VarOp::Raw };
            }
            ':' | '+' | '-' => break,
            _ => {
                name.push(ch);
                chars.next();
            }
        }
    }

    let op = match chars.peek().copied() {
        Some('}') => {
            chars.next();
            VarOp::Raw
        }
        Some(':') if matches!(chars.clone().nth(1), Some('-' | '+')) => {
            chars.next(); // consume ':'
            match chars.next() {
                Some('-') => VarOp::Default {
                    fallback: read_until_brace(chars),
                    treat_empty_as_unset: true,
                },
                Some('+') => VarOp::Alternate {
                    alt: read_until_brace(chars),
                    require_non_empty: true,
                },
                _ => VarOp::Raw,
            }
        }
        Some('-') => {
            chars.next();
            VarOp::Default {
                fallback: read_until_brace(chars),
                treat_empty_as_unset: false,
            }
        }
        Some('+') => {
            chars.next();
            VarOp::Alternate {
                alt: read_until_brace(chars),
                require_non_empty: false,
            }
        }
        _ => VarOp::Raw,
    };

    VarExpr { name, op }
}

fn read_until_brace(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut buf = String::new();
    let mut depth = 0usize;
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                depth += 1;
                buf.push(ch);
            }
            '}' if depth == 0 => break,
            '}' => {
                depth -= 1;
                buf.push(ch);
            }
            other => buf.push(other),
        }
    }
    buf
}

fn is_valid_unbraced_start(ch: char) -> bool {
    is_name_start(ch) || ch.is_ascii_digit()
}

fn is_name_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_name_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

pub(crate) fn restore_literal_dollars(mut s: String) -> String {
    if s.contains(LITERAL_DOLLAR_SENTINEL) {
        s = s.replace(LITERAL_DOLLAR_SENTINEL, "$");
    }
    s
}
