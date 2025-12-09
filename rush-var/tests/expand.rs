use rush_var::env_source::{FnVarSrc, VarSrcChain};
use rush_var::expand_var;
use std::collections::{BTreeMap, HashMap};

#[test]
fn expands_braced_and_unbraced_vars() {
    let mut env = HashMap::new();
    env.insert("FOO".into(), "bar".into());
    env.insert("PATH".into(), "/usr/bin".into());

    // 展开 $FOO 为 bar，${PATH} 为 /usr/bin
    assert_eq!(expand_var("$FOO/${PATH}", &env), "bar//usr/bin");
}

#[test]
fn supports_literal_dollar() {
    let env = HashMap::new();
    // $$ 保留为单个字面 $
    assert_eq!(expand_var("Price $$5", &env), "Price $5");
}

#[test]
fn falls_back_when_unset_or_empty() {
    let mut env = HashMap::new();
    // FOO 未设置，用默认值 /tmp
    assert_eq!(expand_var("${FOO:-/tmp}/bin", &env), "/tmp/bin");

    env.insert("FOO".into(), "".into());
    // FOO 为空，:- 仍使用 fallback
    assert_eq!(expand_var("val=${FOO:-fallback}", &env), "val=fallback");
}

#[test]
fn nested_defaults_expand_once_like_posix_word() {
    let mut env = HashMap::new();
    env.insert("INNER".into(), "x".into());
    // OUTER 未设，默认值里再展开 ${INNER} 得到 x
    assert_eq!(expand_var("${OUTER:-${INNER:-z}}", &env), "x");

    env.remove("INNER");
    env.insert("FALL".into(), "fallback".into());
    // OUTER/INNER 均未设，默认值 word 再展开 $FALL
    assert_eq!(expand_var("${OUTER:-${INNER:-$FALL}}", &env), "fallback");
}

#[test]
fn only_uses_default_when_unset() {
    let mut env = HashMap::new();
    // FOO 未设，- 使用默认值
    assert_eq!(expand_var("val=${FOO-/usr/bin}", &env), "val=/usr/bin");

    env.insert("FOO".into(), "".into());
    // FOO 为空但已设，- 不触发默认
    assert_eq!(expand_var("val=${FOO-/usr/bin}", &env), "val=");
}

#[test]
fn alternate_forms_match_posix_behavior() {
    let mut env = HashMap::new();
    env.insert("SET".into(), "x".into());
    env.insert("EMPTY".into(), "".into());

    // SET 非空，:+ 取 yes
    assert_eq!(expand_var("a=${SET:+yes}", &env), "a=yes");
    // EMPTY 为空，:+ 不展开 alt
    assert_eq!(expand_var("b=${EMPTY:+yes}", &env), "b=");
    // SET 已设，+ 取 ok
    assert_eq!(expand_var("c=${SET+ok}", &env), "c=ok");
    // EMPTY 已设但为空，+ 仍取 ok
    assert_eq!(expand_var("d=${EMPTY+ok}", &env), "d=ok");
    // MISS 未设，+ 不取 alt
    assert_eq!(expand_var("e=${MISS+ok}", &env), "e=");

    env.insert("ALT".into(), "1".into());
    // ALT 触发 :+，内部 ${SET:-zzz} 先尝试 SET（存在且非空）
    assert_eq!(expand_var("f=${ALT:+${SET:-zzz}}", &env), "f=x");
    env.remove("SET");
    // ALT 触发 :+，内部 ${SET:-zzz} 使用默认 zzz
    assert_eq!(expand_var("g=${ALT:+${SET:-zzz}}", &env), "g=zzz");
}

#[test]
fn best_effort_for_unclosed_brace() {
    let mut env = HashMap::new();
    env.insert("FOO".into(), "bar".into());
    // 未闭合 brace 也尽力展开 FOO
    assert_eq!(expand_var("start_${FOO", &env), "start_bar");
}

#[test]
fn recursive_expansion_and_depth_guard() {
    let mut env = HashMap::new();
    env.insert("A".into(), "$B".into());
    env.insert("B".into(), "$C".into());
    env.insert("C".into(), "c".into());
    // 递归展开 A->B->C
    assert_eq!(expand_var("$A-$C", &env), "c-c");

    env.insert("LOOP".into(), "$LOOP".into());
    // 自引用在递归深度上限后原样返回
    assert_eq!(expand_var("$LOOP", &env), "$LOOP");
}

#[test]
fn supports_multiple_env_sources() {
    let primary = [("1", "uno")];
    let mut fallback = HashMap::new();
    fallback.insert("2".into(), "dos".into());
    let chain = VarSrcChain {
        primary: &primary[..],
        fallback: &fallback,
    };
    // 主源取 1，后备取 2，缺失的 3 为空
    assert_eq!(expand_var("$1/$2/$3", &chain), "uno/dos/");

    let func_env = FnVarSrc(|key: &str| if key == "USER" { Some("alice".into()) } else { None });
    // 闭包源返回 USER
    assert_eq!(expand_var("hi_$USER", &func_env), "hi_alice");

    let mut tree_env = BTreeMap::new();
    tree_env.insert("FOO".into(), "bar".into());
    // BTreeMap 源，普通变量替换
    assert_eq!(expand_var("{$FOO}", &tree_env), "{bar}");
}
