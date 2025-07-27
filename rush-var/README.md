# rush-var

[![Crates.io](https://img.shields.io/crates/v/rush-var.svg)](https://crates.io/crates/rush-var)
[![Docs.rs](https://img.shields.io/docsrs/rush-var)](https://docs.rs/rush-var)
[![License](https://img.shields.io/crates/l/rush-var)](LICENSE)

A recursive shell-style variable interpolator for Rust, supporting `$VAR`, `${VAR}`, and `${VAR:-default}` patterns.

## ✨ Features

- ✅ Bash-style variable expansion: `$FOO`, `${FOO}`, `${FOO:-default}`
- ✅ Recursive resolution: values can reference other variables
- ✅ Supports default values via `${VAR:-default}`
- ✅ Fully customizable value source (not bound to `std::env`)
- ✅ Zero unsafe, dependency-light

## 🔧 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rush-var = "0.1"
```

## 🚀 Usage

### Basic interpolation

```rust
use rush_var::expand_env;

let env = [("FOO", "bar")];
assert_eq!(expand_env("Hello $FOO!", &env), "Hello bar!");
assert_eq!(expand_env("path=${BAR:-/usr/local}/bin", &env), "path=/usr/local/bin");
```

### Recursive expansion

```rust
use std::collections::HashMap;
use rush_var::expand_env_recursive;

let mut env = HashMap::new();
env.insert("A".into(), "$B".into());
env.insert("B".into(), "value".into());

assert_eq!(expand_env_recursive("A=$A", &env), "A=value");
```

### Using custom Fn closure as environment

```rust
use rush_var::env_source::FnEnvSource;
use rush_var::expand_env;

let env = FnEnvSource( | key: & str| {
match key {
"USER" => Some("alice".to_string()),
_ => None,
}
});
assert_eq!(expand_env("hi_$USER", &env), "hi_alice");
```

### Chain multiple sources

```rust
use rush_var::env_source::EnvSourceChain;
use rush_var::expand_env;
use std::collections::HashMap;

let main = [("FOO", "123")];
let mut fallback = HashMap::new();
fallback.insert("BAR".to_string(), "456".to_string());

let chain = EnvSourceChain {
primary: & main[..],
fallback: & fallback,
};

assert_eq!(expand_env("$FOO,$BAR", &chain), "123,456");
```

### Expand using std::env::vars()

```rust
use rush_var::expand_env_vars;

std::env::set_var("FOO", "system");
assert_eq!(expand_env_vars(">> $FOO <<"), ">> system <<");
```

## 📘 API

```rust
pub fn expand_env(input: &str, env: &impl EnvSource) -> String
```

- Performs one-pass shell-style variable interpolation
- Supports `$VAR`, `${VAR}`, `${VAR:-default}`, `$$`
- `env` can be any source implementing `EnvSource` trait (e.g., `HashMap`, slice, closure, etc.)

```rust
pub fn expand_env_recursive(input: &str, env: &impl EnvSource) -> String
```

- Performs recursive interpolation, expanding variables up to 8 layers deep
- Recommended when variable values may also contain interpolations

```rust
pub fn expand_env_vars(input: &str) -> String
```

- Uses `std::env::vars()` as the environment source
- Equivalent to: `expand_env_recursive(input, &std::env::vars())`

## 💡 Supported Syntax

| Syntax                | Meaning                                                     |
|-----------------------|-------------------------------------------------------------|
| `$VAR`                | Expand variable `VAR` if present, else empty string         |
| `${VAR}`              | Same as `$VAR`                                              |
| `${VAR:-default}`     | Use `default` if `VAR` is undefined                         |
| `$$`                  | Literal dollar sign `$`                                     |
| `$VAR/$UNKNOWN`       | Unknown variable expands to empty string                    |
| `${VAR:-/path}`       | Default value can include any characters, even `/`          |
| `${FOO:-$BAR}`        | Default itself can contain variables (recursively expanded) |
| `$VAR_with_trailing!` | Stops at first non-alphanumeric/underscore character        |

### 🚫 Not Supported (yet)

| Syntax            | Status | Notes                      |
|-------------------|--------|----------------------------|
| `${VAR:+alt}`     | ❌      | Alternate value if defined |
| `${#VAR}`         | ❌      | Length of value            |
| `${VAR/sub/repl}` | ❌      | Substring replacement      |

## 🛡️ Safety

- Recursion depth is limited to avoid infinite loops.
- Invalid syntax returns a structured `Error`.

## 📄 License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).

---

> Made with ❤️ by [BppleMan]
