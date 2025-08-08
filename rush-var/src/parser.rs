use crate::ast::*;
use crate::lexer::{parse_int, read_until_unescaped};

pub fn parse_braced(source: &str) -> Result<ParamExpr, Error> {
    if !source.starts_with("${") || !source.ends_with('}') {
        return Err(Error::BadSubstitution("missing braces".into()));
    }
    let inner = &source[2..source.len() - 1];
    parse_inner(inner)
}

fn parse_inner(mut s: &str) -> Result<ParamExpr, Error> {
    let mut length = false;
    if let Some(rest) = s.strip_prefix('#') {
        length = true;
        s = rest;
    }
    let mut indirection = false;
    if let Some(rest) = s.strip_prefix('!') {
        indirection = true;
        s = rest;
    }

    // parse name
    let mut name = String::new();
    let mut idx = 0;
    for ch in s.chars() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            name.push(ch);
            idx += ch.len_utf8();
        } else {
            break;
        }
    }
    if name.is_empty() {
        return Err(Error::BadSubstitution("missing name".into()));
    }
    s = &s[idx..];
    let mut expr = ParamExpr::Ref {
        target: Target { name },
    };

    if s.is_empty() {
        // no more
    } else if let Some(rest) = s.strip_prefix(':') {
        // could be defaulting or substring
        if let Some(op) = rest.chars().next() {
            if op == '-' || op == '+' || op == '=' || op == '?' {
                let word = rest[1..].to_string();
                let op = match op {
                    '-' => DefaultOp::Dash,
                    '+' => DefaultOp::Plus,
                    '=' => DefaultOp::Assign,
                    '?' => DefaultOp::QMark,
                    _ => unreachable!(),
                };
                expr = ParamExpr::Defaulting {
                    inner: Box::new(expr),
                    colon: true,
                    op,
                    word,
                };
                s = "";
            } else {
                // substring
                let (off, rest2) = parse_int(rest);
                let mut len = None;
                let mut tail = rest2;
                if let Some(r) = tail.strip_prefix(':') {
                    let (l, r2) = parse_int(r);
                    len = Some(l);
                    tail = r2;
                }
                expr = ParamExpr::Substring {
                    inner: Box::new(expr),
                    offset: off,
                    len,
                };
                s = tail;
            }
        } else {
            s = "";
        }
    } else if let Some(rest) = s.strip_prefix('-') {
        expr = ParamExpr::Defaulting {
            inner: Box::new(expr),
            colon: false,
            op: DefaultOp::Dash,
            word: rest.to_string(),
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix('+') {
        expr = ParamExpr::Defaulting {
            inner: Box::new(expr),
            colon: false,
            op: DefaultOp::Plus,
            word: rest.to_string(),
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix('=') {
        expr = ParamExpr::Defaulting {
            inner: Box::new(expr),
            colon: false,
            op: DefaultOp::Assign,
            word: rest.to_string(),
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix('?') {
        expr = ParamExpr::Defaulting {
            inner: Box::new(expr),
            colon: false,
            op: DefaultOp::QMark,
            word: rest.to_string(),
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix("##") {
        let pattern = rest.to_string();
        expr = ParamExpr::Remove {
            inner: Box::new(expr),
            op: RemoveOp::Prefix { long: true },
            pattern,
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix('#') {
        let pattern = rest.to_string();
        expr = ParamExpr::Remove {
            inner: Box::new(expr),
            op: RemoveOp::Prefix { long: false },
            pattern,
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix("%%") {
        let pattern = rest.to_string();
        expr = ParamExpr::Remove {
            inner: Box::new(expr),
            op: RemoveOp::Suffix { long: true },
            pattern,
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix('%') {
        let pattern = rest.to_string();
        expr = ParamExpr::Remove {
            inner: Box::new(expr),
            op: RemoveOp::Suffix { long: false },
            pattern,
        };
        s = "";
    } else if let Some(rest) = s.strip_prefix("//") {
        let (pat, rest) = read_until_unescaped(rest, '/');
        let (repl, tail) = read_until_unescaped(rest, '\0');
        expr = ParamExpr::Replace {
            inner: Box::new(expr),
            scope: ReplaceScope::Global,
            pat,
            repl,
        };
        s = tail;
    } else if let Some(rest) = s.strip_prefix("/#") {
        let (pat, rest) = read_until_unescaped(rest, '/');
        let (repl, tail) = read_until_unescaped(rest, '\0');
        expr = ParamExpr::Replace {
            inner: Box::new(expr),
            scope: ReplaceScope::AnchorPrefix,
            pat,
            repl,
        };
        s = tail;
    } else if let Some(rest) = s.strip_prefix("/%") {
        let (pat, rest) = read_until_unescaped(rest, '/');
        let (repl, tail) = read_until_unescaped(rest, '\0');
        expr = ParamExpr::Replace {
            inner: Box::new(expr),
            scope: ReplaceScope::AnchorSuffix,
            pat,
            repl,
        };
        s = tail;
    } else if let Some(rest) = s.strip_prefix('/') {
        let (pat, rest) = read_until_unescaped(rest, '/');
        let (repl, tail) = read_until_unescaped(rest, '\0');
        expr = ParamExpr::Replace {
            inner: Box::new(expr),
            scope: ReplaceScope::First,
            pat,
            repl,
        };
        s = tail;
    }

    if !s.is_empty() {
        return Err(Error::BadSubstitution(format!("trailing characters: {}", s)));
    }

    if indirection {
        expr = ParamExpr::Indirection { inner: Box::new(expr) };
    }
    if length {
        expr = ParamExpr::Length { inner: Box::new(expr) };
    }
    Ok(expr)
}
