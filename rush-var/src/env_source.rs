use std::collections::{BTreeMap, HashMap};

/// 变量查找源 trait。用于支持多种来源的变量查找。
/// Trait for abstracting environment variable lookup source.
///
/// # 用法示例
/// ```rust
/// use rush_var::expand_var;
/// let env = [ ("FOO", "bar") ];
/// let res = expand_var("$FOO", &env);
/// assert_eq!(res, "bar");
/// ```
pub trait VarSrc {
    /// 获取指定 key 的变量值，如果不存在则返回 None。
    fn get(&self, key: &str) -> Option<String>;
}

/// 为任意已实现 VarSrc 的类型的引用自动实现 VarSrc
impl<T: VarSrc + ?Sized> VarSrc for &T {
    fn get(&self, key: &str) -> Option<String> {
        (**self).get(key)
    }
}

/// HashMap 作为环境变量源
impl VarSrc for HashMap<String, String> {
    fn get(&self, key: &str) -> Option<String> {
        self.get(key).cloned()
    }
}

/// BTreeMap 作为环境变量源
impl VarSrc for BTreeMap<String, String> {
    fn get(&self, key: &str) -> Option<String> {
        self.get(key).cloned()
    }
}

/// 切片 &[(&str, &str)] 作为环境变量源，适用于快速mock和常量环境。
impl<'a> VarSrc for &'a [(&'a str, &'a str)] {
    fn get(&self, key: &str) -> Option<String> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| (*v).to_owned())
    }
}

/// 固定长度数组 [(&str, &str); N] 作为环境变量源，便于简单用例或测试。
impl<'a, const N: usize> VarSrc for [(&'a str, &'a str); N] {
    fn get(&self, key: &str) -> Option<String> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| (*v).to_owned())
    }
}

/// 系统环境变量（字符串）
impl VarSrc for std::env::Vars {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// 系统环境变量（OsString）
impl VarSrc for std::env::VarsOs {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var_os(key).and_then(|v| v.into_string().ok())
    }
}

/// 闭包适配器，允许以自定义函数/闭包方式提供变量查找逻辑。
///
/// # 用法示例
/// ```rust
/// use rush_var::env_source::FnVarSrc;
/// use rush_var::expand_var;
/// let env = FnVarSrc(|key: &str| if key == "FOO" { Some("baz".into()) } else { None });
/// assert_eq!(expand_var("abc$FOO", &env), "abcbaz");
/// ```
pub struct FnVarSrc<F>(pub F);

impl<F> VarSrc for FnVarSrc<F>
where
    for<'a> F: Fn(&'a str) -> Option<String>,
{
    fn get(&self, key: &str) -> Option<String> {
        self.0(key)
    }
}

/// 链式环境变量源：优先查询 primary，没有再查 fallback。
/// 常用于“临时变量+系统变量”的多层环境方案。
pub struct VarSrcChain<A, B> {
    /// 主查找源（优先）
    pub primary: A,
    /// 备选查找源（兜底）
    pub fallback: B,
}

impl<A: VarSrc, B: VarSrc> VarSrc for VarSrcChain<A, B> {
    fn get(&self, key: &str) -> Option<String> {
        self.primary.get(key).or_else(|| self.fallback.get(key))
    }
}

/// 兼容旧命名，保留别名
pub trait EnvSource: VarSrc {}
impl<T: VarSrc + ?Sized> EnvSource for T {}
pub type FnEnvSource<F> = FnVarSrc<F>;
pub type EnvSourceChain<A, B> = VarSrcChain<A, B>;
