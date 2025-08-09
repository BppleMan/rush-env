//! # rush-var
//!
//! A Rust library for parsing and evaluating shell parameter expansions with zsh-first semantics.
//!
//! This library provides a complete implementation of shell parameter expansion including:
//! - Basic variable references: `$name`, `${name}`
//! - Length operations: `${#name}`
//! - Default/assign/alt/error operations: `${name:-default}`, `${name:=default}`, etc.
//! - Prefix/suffix removal: `${name#pattern}`, `${name%pattern}`
//! - String replacement: `${name/pattern/replacement}`
//! - Substring extraction: `${name:offset:length}`
//! - Indirection: `${!name}`, `${(P)name}`
//! - Zsh parameter expansion flags: `${(flags)name}`
//! - Path modifiers: `${name:h:t:r:e}`
//! - Array and associative array support
//!
//! ## Example
//!
//! ```rust
//! use rush_var::{expand_str, Env, Options};
//!
//! let mut env = Env::new();
//! env.set_scalar("USER", "alice");
//! env.set_scalar("HOME", "/home/alice");
//!
//! let options = Options::default();
//! let result = expand_str("Hello ${USER}, your home is ${HOME}", &env, &options)?;
//! assert_eq!(result, "Hello alice, your home is /home/alice");
//! # Ok::<(), rush_var::Error>(())
//! ```

pub mod ast;
pub mod env;
pub mod lexer;
pub mod parser;
pub mod eval;

// Re-export main types and functions
pub use ast::*;
pub use env::{Env, Value};
pub use eval::{GlobKind, Mode, Options, evaluate_expr, expand_str};
pub use parser::parse_braced;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_test_env() -> Env {
        let mut env = Env::new();
        env.set_scalar("FOO", "bar");
        env.set_scalar("EMPTY", "");
        env.set_scalar("PATH", "/usr/bin:/bin");
        env.set_scalar("HOME", "/home/user");
        env.set_scalar("USER", "testuser");
        env.set_array("ARR", vec!["one", "two", "three"]);
        env.set_assoc("MAP", vec![("key1", "val1"), ("key2", "val2")]);
        env.set_positional(vec!["arg1", "arg2", "arg3"]);
        env
    }

    fn test_options() -> Options {
        Options {
            mode: Mode::Zsh,
            allow_exec_subst: false,
            allow_flag_e: false,
            glob_impl: GlobKind::Simple,
        }
    }

    #[test]
    fn test_basic_variable_expansion() {
        let env = create_test_env();
        let opts = test_options();

        // Basic variable references
        assert_eq!(expand_str("$FOO", &env, &opts).unwrap(), "bar");
        assert_eq!(expand_str("${FOO}", &env, &opts).unwrap(), "bar");
        assert_eq!(expand_str("pre_${FOO}_post", &env, &opts).unwrap(), "pre_bar_post");
        assert_eq!(expand_str("${FOO}bar", &env, &opts).unwrap(), "barbar");

        // Multiple expansions
        assert_eq!(expand_str("$FOO-$USER", &env, &opts).unwrap(), "bar-testuser");
        assert_eq!(expand_str("${FOO}_${USER}", &env, &opts).unwrap(), "bar_testuser");
    }

    #[test]
    fn test_special_parameters() {
        let env = create_test_env();
        let opts = test_options();

        // Special parameters
        assert_eq!(expand_str("${$}", &env, &opts).unwrap(), "12345"); // PID
        assert_eq!(expand_str("${?}", &env, &opts).unwrap(), "0"); // Last status
        assert_eq!(expand_str("${0}", &env, &opts).unwrap(), "zsh"); // Shell name
        assert_eq!(expand_str("${#}", &env, &opts).unwrap(), "3"); // Positional count
    }

    #[test]
    fn test_positional_parameters() {
        let env = create_test_env();
        let opts = test_options();

        // Positional parameters
        assert_eq!(expand_str("${1}", &env, &opts).unwrap(), "arg1");
        assert_eq!(expand_str("${2}", &env, &opts).unwrap(), "arg2");
        assert_eq!(expand_str("${3}", &env, &opts).unwrap(), "arg3");
        assert_eq!(expand_str("${10}", &env, &opts).unwrap(), ""); // Out of range
    }

    #[test]
    fn test_length_operation() {
        let env = create_test_env();
        let opts = test_options();

        // Length operation
        assert_eq!(expand_str("${#FOO}", &env, &opts).unwrap(), "3");
        assert_eq!(expand_str("${#EMPTY}", &env, &opts).unwrap(), "0");
        assert_eq!(expand_str("${#PATH}", &env, &opts).unwrap(), "13");
        assert_eq!(expand_str("${#ARR}", &env, &opts).unwrap(), "3"); // Array first element length
    }

    #[test]
    fn test_default_operations() {
        let env = create_test_env();
        let opts = test_options();

        // Default with dash (colon variants)
        assert_eq!(expand_str("${FOO:-default}", &env, &opts).unwrap(), "bar");
        assert_eq!(expand_str("${EMPTY:-default}", &env, &opts).unwrap(), "default");
        assert_eq!(expand_str("${UNSET:-default}", &env, &opts).unwrap(), "default");

        // Default with dash (non-colon variants)
        assert_eq!(expand_str("${FOO-default}", &env, &opts).unwrap(), "bar");
        assert_eq!(expand_str("${EMPTY-default}", &env, &opts).unwrap(), ""); // Set but empty
        assert_eq!(expand_str("${UNSET-default}", &env, &opts).unwrap(), "default");

        // Plus operation
        assert_eq!(expand_str("${FOO:+alt}", &env, &opts).unwrap(), "alt");
        assert_eq!(expand_str("${EMPTY:+alt}", &env, &opts).unwrap(), "");
        assert_eq!(expand_str("${UNSET:+alt}", &env, &opts).unwrap(), "");

        assert_eq!(expand_str("${FOO+alt}", &env, &opts).unwrap(), "alt");
        assert_eq!(expand_str("${EMPTY+alt}", &env, &opts).unwrap(), "alt"); // Set but empty
        assert_eq!(expand_str("${UNSET+alt}", &env, &opts).unwrap(), "");
    }

    #[test]
    fn test_error_operation() {
        let env = create_test_env();
        let opts = test_options();

        // Question mark operation (should succeed)
        assert_eq!(expand_str("${FOO:?error}", &env, &opts).unwrap(), "bar");
        assert_eq!(expand_str("${FOO?error}", &env, &opts).unwrap(), "bar");

        // Question mark operation (should error)
        assert!(expand_str("${UNSET:?error message}", &env, &opts).is_err());
        assert!(expand_str("${EMPTY:?error message}", &env, &opts).is_err());
        assert!(expand_str("${UNSET?error message}", &env, &opts).is_err());
    }

    #[test]
    fn test_prefix_suffix_removal() {
        let env = create_test_env();
        let opts = test_options();

        // Test with PATH-like variable
        assert_eq!(expand_str("${PATH#*/}", &env, &opts).unwrap(), "usr/bin:/bin");
        assert_eq!(expand_str("${PATH##*/}", &env, &opts).unwrap(), "bin");
        assert_eq!(expand_str("${PATH%:*}", &env, &opts).unwrap(), "/usr/bin");
        assert_eq!(expand_str("${PATH%%:*}", &env, &opts).unwrap(), "/usr/bin");

        // Test with filename-like string
        let mut env2 = create_test_env();
        env2.set_scalar("FILE", "path/to/file.txt");

        assert_eq!(expand_str("${FILE#*/}", &env2, &opts).unwrap(), "to/file.txt");
        assert_eq!(expand_str("${FILE##*/}", &env2, &opts).unwrap(), "file.txt");
        assert_eq!(expand_str("${FILE%.*}", &env2, &opts).unwrap(), "path/to/file");
        assert_eq!(expand_str("${FILE%%.*}", &env2, &opts).unwrap(), "path/to/file");
    }

    #[test]
    fn test_string_replacement() {
        let mut env2 = create_test_env();
        env2.set_scalar("TEXT", "hello world hello");
        let opts = test_options();

        // Replace operations
        assert_eq!(expand_str("${TEXT/hello/hi}", &env2, &opts).unwrap(), "hi world hello");
        assert_eq!(expand_str("${TEXT//hello/hi}", &env2, &opts).unwrap(), "hi world hi");

        // Anchored replacements
        env2.set_scalar("TEXT2", "hello world");
        assert_eq!(expand_str("${TEXT2/#hello/hi}", &env2, &opts).unwrap(), "hi world");
        assert_eq!(expand_str("${TEXT2/%world/universe}", &env2, &opts).unwrap(), "hello universe");
    }

    #[test]
    fn test_substring_extraction() {
        let mut env2 = create_test_env();
        env2.set_scalar("TEXT", "hello world");
        let opts = test_options();

        // Substring operations
        assert_eq!(expand_str("${TEXT:0:5}", &env2, &opts).unwrap(), "hello");
        assert_eq!(expand_str("${TEXT:6}", &env2, &opts).unwrap(), "world");
        assert_eq!(expand_str("${TEXT:6:3}", &env2, &opts).unwrap(), "wor");
        assert_eq!(expand_str("${TEXT:-5}", &env2, &opts).unwrap(), "world"); // Negative offset
    }

    #[test]
    fn test_indirection() {
        let mut env = create_test_env();
        env.set_scalar("VAR", "FOO");
        let opts = test_options();

        // Indirection operations
        assert_eq!(expand_str("${!VAR}", &env, &opts).unwrap(), "bar");
        // Note: ${(P)VAR} would be tested with zsh flags
    }

    #[test]
    fn test_array_operations() {
        let env = create_test_env();
        let opts = test_options();

        // Array indexing
        assert_eq!(expand_str("${ARR[1]}", &env, &opts).unwrap(), "one");
        assert_eq!(expand_str("${ARR[2]}", &env, &opts).unwrap(), "two");
        assert_eq!(expand_str("${ARR[3]}", &env, &opts).unwrap(), "three");

        // Array slicing
        assert_eq!(expand_str("${ARR[1,2]}", &env, &opts).unwrap(), "one");
        assert_eq!(expand_str("${ARR[2,3]}", &env, &opts).unwrap(), "two");

        // Array length
        assert_eq!(expand_str("${#ARR}", &env, &opts).unwrap(), "3");
    }

    #[test]
    fn test_associative_array_operations() {
        let env = create_test_env();
        let opts = test_options();

        // Associative array access
        assert_eq!(expand_str("${MAP[key1]}", &env, &opts).unwrap(), "val1");
        assert_eq!(expand_str("${MAP[key2]}", &env, &opts).unwrap(), "val2");
        assert_eq!(expand_str("${MAP[nonexistent]}", &env, &opts).unwrap(), "");
    }

    #[test]
    fn test_zsh_flags() {
        let mut env2 = create_test_env();
        env2.set_scalar("TEXT", "Hello World");
        env2.set_scalar("LOWER", "hello world");
        let opts = test_options();

        // Case conversion flags
        assert_eq!(expand_str("${(U)LOWER}", &env2, &opts).unwrap(), "HELLO WORLD");
        assert_eq!(expand_str("${(L)TEXT}", &env2, &opts).unwrap(), "hello world");

        // Quote flag
        env2.set_scalar("SPACED", "hello world");
        assert_eq!(expand_str("${(q)SPACED}", &env2, &opts).unwrap(), "'hello world'");

        // Split flag
        env2.set_scalar("CSV", "a,b,c");
        assert_eq!(expand_str("${(s:,:)CSV}", &env2, &opts).unwrap(), "a b c");

        // Join flag for arrays
        let env = create_test_env();
        assert_eq!(expand_str("${(j:,:)ARR}", &env, &opts).unwrap(), "one,two,three");
    }

    #[test]
    fn test_path_modifiers() {
        let mut env2 = create_test_env();
        env2.set_scalar("FILEPATH", "/path/to/file.txt");
        let opts = test_options();

        // Path modifiers
        assert_eq!(expand_str("${FILEPATH:h}", &env2, &opts).unwrap(), "/path/to");
        assert_eq!(expand_str("${FILEPATH:t}", &env2, &opts).unwrap(), "file.txt");
        assert_eq!(expand_str("${FILEPATH:r}", &env2, &opts).unwrap(), "/path/to/file");
        assert_eq!(expand_str("${FILEPATH:e}", &env2, &opts).unwrap(), "txt");

        // Chained modifiers
        assert_eq!(expand_str("${FILEPATH:t:r}", &env2, &opts).unwrap(), "file");
    }

    #[test]
    fn test_nested_expansions() {
        let mut env = create_test_env();
        env.set_scalar("VAR1", "FOO");
        env.set_scalar("VAR2", "default_value");
        let opts = test_options();

        // Nested expansions in word
        assert_eq!(expand_str("${FOO:-${VAR2}}", &env, &opts).unwrap(), "bar");
        assert_eq!(expand_str("${UNSET:-${VAR2}}", &env, &opts).unwrap(), "default_value");

        // Nested in pattern/replacement
        env.set_scalar("PATTERN", "o");
        env.set_scalar("REPL", "X");
        env.set_scalar("TARGET", "hello");
        assert_eq!(expand_str("${TARGET/${PATTERN}/${REPL}}", &env, &opts).unwrap(), "hellX");
    }

    #[test]
    fn test_complex_scenarios() {
        let mut env = create_test_env();
        env.set_scalar("CONFIG_FILE", "/etc/app/config.json");
        env.set_scalar("BACKUP_DIR", "/backup");
        env.set_scalar("DATE", "2023-12-01");
        let opts = test_options();

        // Complex real-world example
        let backup_path = "${BACKUP_DIR}/${CONFIG_FILE:t:r}_${DATE}.bak";
        assert_eq!(expand_str(backup_path, &env, &opts).unwrap(), "/backup/config_2023-12-01.bak");

        // Multiple operations
        env.set_scalar("URL", "https://example.com/path/file.html");
        assert_eq!(expand_str("${URL#*://}", &env, &opts).unwrap(), "example.com/path/file.html");
        assert_eq!(expand_str("${URL##*/}", &env, &opts).unwrap(), "file.html");
        assert_eq!(expand_str("${URL%.*}", &env, &opts).unwrap(), "https://example.com/path/file");
    }

    #[test]
    fn test_error_cases() {
        let env = create_test_env();
        let opts = test_options();

        // Malformed expansions
        assert!(parse_braced("${FOO").is_err()); // Unclosed brace
        assert!(parse_braced("${FOO:}").is_err()); // Invalid operation

        // Bad substitution errors should be caught during parsing
        assert!(expand_str("${}", &env, &opts).is_err());
    }

    #[test]
    fn test_env_operations() {
        let mut env = Env::new();

        // Test environment operations
        env.set_scalar("TEST", "value");
        assert!(env.is_set("TEST"));
        assert!(env.is_set_and_non_empty("TEST"));
        assert_eq!(env.get("TEST").unwrap().to_scalar(), "value");

        env.set_scalar("EMPTY", "");
        assert!(env.is_set("EMPTY"));
        assert!(!env.is_set_and_non_empty("EMPTY"));

        env.unset("TEST");
        assert!(!env.is_set("TEST"));

        // Test value operations
        let scalar = Value::scalar("test");
        assert_eq!(scalar.len(), 4);
        assert!(!scalar.is_empty());

        let array = Value::array(vec!["a", "b", "c"]);
        assert_eq!(array.len(), 3);
        assert_eq!(array.to_scalar(), "a b c");

        let empty_array = Value::array(Vec::<String>::new());
        assert!(empty_array.is_empty());
        assert_eq!(empty_array.len(), 0);
    }

    #[test]
    fn test_glob_matching() {
        use crate::eval::glob_match;

        // Basic glob patterns
        assert!(glob_match("hello", "hello"));
        assert!(!glob_match("hello", "world"));

        // Wildcard patterns
        assert!(glob_match("h*", "hello"));
        assert!(glob_match("h*o", "hello"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*.*", "file.txt"));

        // Question mark patterns
        assert!(glob_match("h?llo", "hello"));
        assert!(glob_match("h?llo", "hallo"));
        assert!(!glob_match("h?llo", "hllo"));

        // Character classes
        assert!(glob_match("h[aei]llo", "hello"));
        assert!(glob_match("h[aei]llo", "hallo"));
        assert!(!glob_match("h[aei]llo", "hollo"));
    }

    #[test]
    fn test_parser_edge_cases() {
        let env = create_test_env();
        let opts = test_options();

        // Test parsing edge cases
        assert_eq!(expand_str("${FOO}", &env, &opts).unwrap(), "bar");
        assert_eq!(expand_str("$FOO", &env, &opts).unwrap(), "bar");

        // Mixed literal and expansion
        assert_eq!(expand_str("prefix${FOO}suffix", &env, &opts).unwrap(), "prefixbarsuffix");
        assert_eq!(expand_str("${FOO}${USER}", &env, &opts).unwrap(), "bartestuser");

        // No expansion
        assert_eq!(expand_str("no expansion here", &env, &opts).unwrap(), "no expansion here");
        assert_eq!(expand_str("literal $ dollar", &env, &opts).unwrap(), "literal $ dollar");
    }

    #[test]
    fn test_options_modes() {
        let env = create_test_env();

        // Test different shell modes
        let zsh_opts = Options {
            mode: Mode::Zsh,
            ..Default::default()
        };
        let bash_opts = Options {
            mode: Mode::Bash,
            ..Default::default()
        };
        let posix_opts = Options {
            mode: Mode::Posix,
            ..Default::default()
        };

        // Basic functionality should work in all modes
        assert_eq!(expand_str("${FOO}", &env, &zsh_opts).unwrap(), "bar");
        assert_eq!(expand_str("${FOO}", &env, &bash_opts).unwrap(), "bar");
        assert_eq!(expand_str("${FOO}", &env, &posix_opts).unwrap(), "bar");
    }

    #[test]
    fn test_security_options() {
        let env = create_test_env();

        // Test security-sensitive options
        let safe_opts = Options {
            allow_exec_subst: false,
            allow_flag_e: false,
            ..Default::default()
        };
        let _unsafe_opts = Options {
            allow_exec_subst: true,
            allow_flag_e: true,
            ..Default::default()
        };

        // Command substitution should be disabled by default
        assert!(expand_str("$(echo test)", &env, &safe_opts).unwrap().contains("$(echo test)"));

        // (e) flag should be disabled by default
        let mut test_env = create_test_env();
        test_env.set_scalar("EXPAND_ME", "$FOO");
        assert!(expand_str("${(e)EXPAND_ME}", &test_env, &safe_opts).is_err());
    }
}
