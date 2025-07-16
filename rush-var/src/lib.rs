//! A minimal Bash-style env expander crate

use std::collections::HashMap;

/// Trait for abstracting environment variable source
pub trait EnvSource {
    fn get(&self, key: &str) -> Option<String>;
}

impl EnvSource for std::env::Vars {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

impl EnvSource for std::env::VarsOs {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var_os(key).and_then(|v| v.into_string().ok())
    }
}

impl EnvSource for HashMap<String, String> {
    fn get(&self, key: &str) -> Option<String> {
        self.get(key).cloned()
    }
}

/// Expands environment variable references in the input string
/// Supports: $VAR, ${VAR}, ${VAR:-default}, $$
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
}
