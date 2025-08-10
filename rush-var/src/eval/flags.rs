use super::ExpansionOptions as Options;
use crate::ast::ZshFlag;
use crate::env::EnvVars;

/// 应用单个 zsh flag 到字符串值
pub(crate) fn apply_zsh_flag<E: EnvVars + ?Sized>(
    value: &str,
    flag: &ZshFlag,
    env: &E,
    opt: &Options,
) -> Result<String, crate::ast::Error> {
    use crate::ast::ZFlag;

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
            if value.chars().any(|c| c.is_whitespace()) {
                Ok(format!("'{}'", value.replace('\'', "'\\''")))
            } else {
                Ok(value.to_string())
            }
        }
        ZFlag::Unquote => {
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
        ZFlag::F => Ok(value.lines().collect::<Vec<_>>().join(" ")),
        ZFlag::Z | ZFlag::ZExt => Ok(value.split_whitespace().collect::<Vec<_>>().join(" ")),
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
        ZFlag::K | ZFlag::V | ZFlag::T => Ok(value.to_string()),
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
        ZFlag::J { sep: _ } => Ok(value.to_string()),
        ZFlag::S { sep } => Ok(value.split(sep).collect::<Vec<_>>().join(" ")),
        ZFlag::VDisplay => Ok(value
            .chars()
            .map(|c| match c {
                '\n' => "\\n".to_string(),
                '\t' => "\\t".to_string(),
                '\r' => "\\r".to_string(),
                '\\' => "\\\\".to_string(),
                _ => c.to_string(),
            })
            .collect()),
        ZFlag::P => {
            if let Some(val) = env.get_var(value) {
                Ok(val.to_scalar())
            } else {
                Ok(String::new())
            }
        }
        ZFlag::E => {
            if opt.allow_flag_e {
                // 重新展开结果
                crate::eval::expand_str(value, env, opt)
            } else {
                Err(crate::ast::Error::Unsupported("(e) flag disabled for security".to_string()))
            }
        }
    }
}
