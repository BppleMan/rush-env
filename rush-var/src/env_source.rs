use std::collections::{BTreeMap, HashMap};

/// 环境变量查找源 trait。用于支持多种来源的变量查找。
/// Trait for abstracting environment variable lookup source.
///
/// # 用法示例
/// ```rust
/// use rush_var::expand_env;
/// let env = [ ("FOO", "bar") ];
/// let res = expand_env("$FOO", &env);
/// assert_eq!(res, "bar");
/// ```
pub trait EnvSource {
    /// 获取指定 key 的变量值，如果不存在则返回 None。
    fn get(&self, key: &str) -> Option<String>;
}

/// 为任意已实现 EnvSource 的类型的引用自动实现 EnvSource
impl<T: EnvSource + ?Sized> EnvSource for &T {
    fn get(&self, key: &str) -> Option<String> {
        (**self).get(key)
    }
}

/// HashMap 作为环境变量源
impl EnvSource for HashMap<String, String> {
    fn get(&self, key: &str) -> Option<String> {
        self.get(key).cloned()
    }
}

/// BTreeMap 作为环境变量源
impl EnvSource for BTreeMap<String, String> {
    fn get(&self, key: &str) -> Option<String> {
        self.get(key).cloned()
    }
}

/// 切片 &[(&str, &str)] 作为环境变量源，适用于快速mock和常量环境。
impl<'a> EnvSource for &'a [(&'a str, &'a str)] {
    fn get(&self, key: &str) -> Option<String> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| (*v).to_owned())
    }
}

/// 系统环境变量（字符串）
impl EnvSource for std::env::Vars {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// 系统环境变量（OsString）
impl EnvSource for std::env::VarsOs {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var_os(key).and_then(|v| v.into_string().ok())
    }
}

/// 闭包适配器，允许以自定义函数/闭包方式提供变量查找逻辑。
///
/// # 用法示例
/// ```rust
/// use rush_var::env_source::{FnEnvSource};
/// use rush_var::expand_env;
/// let env = FnEnvSource(|key: &str| if key == "FOO" { Some("baz".into()) } else { None });
/// assert_eq!(expand_env("abc$FOO", &env), "abcbaz");
/// ```
pub struct FnEnvSource<F>(pub F);

impl<F> EnvSource for FnEnvSource<F>
where
    for<'a> F: Fn(&'a str) -> Option<String>,
{
    fn get(&self, key: &str) -> Option<String> {
        self.0(key)
    }
}

/// 链式环境变量源：优先查询 primary，没有再查 fallback。
/// 常用于“临时变量+系统变量”的多层环境方案。
pub struct EnvSourceChain<A, B> {
    /// 主查找源（优先）
    pub primary: A,
    /// 备选查找源（兜底）
    pub fallback: B,
}

impl<A: EnvSource, B: EnvSource> EnvSource for EnvSourceChain<A, B> {
    fn get(&self, key: &str) -> Option<String> {
        self.primary.get(key).or_else(|| self.fallback.get(key))
    }
}
