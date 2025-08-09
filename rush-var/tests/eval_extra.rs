use rush_var::{Env, Options, expand_str};

fn opts() -> Options {
    Options::default()
}

#[test]
fn join_flag_array_and_assoc_then_more_flags() {
    let mut env = Env::new();
    env.set_array("ARR", vec!["a", "b", "c"]);
    env.set_assoc("MAP", vec![("k1", "v1"), ("k2", "v2")]);

    // ${(j:|:)ARR} -> a|b|c
    assert_eq!(expand_str("${(j:|:)ARR}", &env, &opts()).unwrap(), "a|b|c");

    // ${(Uj:|:)ARR} -> 先 join，再 U
    assert_eq!(expand_str("${(Uj:|:)ARR}", &env, &opts()).unwrap(), "A|B|C");

    // ${(j:,:)MAP} -> 关联数组按值 join
    let out = expand_str("${(j:,:)MAP}", &env, &opts()).unwrap();
    assert!(out.contains("v1") && out.contains("v2") && out.contains(","));
}

#[test]
fn p_and_e_flags_and_exec_subst_path() {
    let mut env = Env::new();
    env.set_scalar("FOO", "bar");
    env.set_scalar("VAR_NAME", "FOO");

    // (P): 间接取值
    assert_eq!(expand_str("${(P)VAR_NAME}", &env, &opts()).unwrap(), "bar");

    // (e): 关闭时报 Unsupported
    let mut o = Options::default();
    o.allow_flag_e = false;
    assert!(expand_str("${(e)FOO}", &env, &o).is_err());

    // (e): 开启时进行二次展开
    o.allow_flag_e = true;
    env.set_scalar("WRAP", "${FOO}");
    assert_eq!(expand_str("${(e)WRAP}", &env, &o).unwrap(), "bar");

    // $(...) 在 allow_exec_subst=false 时保留字面量
    let o2 = Options {
        allow_exec_subst: false,
        ..Options::default()
    };
    assert_eq!(expand_str("$(echo hi)", &env, &o2).unwrap(), "$(echo hi)");
}

#[test]
fn padding_and_unquote_split_variants() {
    let mut env = Env::new();
    env.set_scalar("S", "7");

    // (l)/(r) 使用单参形式，当前实现仅支持空格填充
    assert_eq!(expand_str("${(l:5:)S}", &env, &opts()).unwrap(), "7    ");
    assert_eq!(expand_str("${(r:5:)S}", &env, &opts()).unwrap(), "    7");

    env.set_scalar("Q1", "'hello'");
    env.set_scalar("Q2", "\"world\"");
    assert_eq!(expand_str("${(Q)Q1}", &env, &opts()).unwrap(), "hello");
    assert_eq!(expand_str("${(Q)Q2}", &env, &opts()).unwrap(), "world");

    env.set_scalar("LINES", "a\nb\nc");
    assert_eq!(expand_str("${(f)LINES}", &env, &opts()).unwrap(), "a b c");

    env.set_scalar("WS", " a\t b  c\n");
    assert_eq!(expand_str("${(z)WS}", &env, &opts()).unwrap(), "a b c");
}

#[test]
fn apply_index_strslice_and_scalar_slice() {
    let mut env = Env::new();
    env.set_scalar("FOO", "bar");

    // 标量切片 via substring 语义
    assert_eq!(expand_str("${FOO:0:2}", &env, &opts()).unwrap(), "ba");
    assert_eq!(expand_str("${FOO:10:2}", &env, &opts()).unwrap(), "");
}

#[test]
fn glob_and_pathmods_edge_cases() {
    let mut env = Env::new();
    env.set_scalar("E", "");
    env.set_scalar("BRKT", "test[");

    // glob: 未闭合 '[' 作为字面量
    use rush_var::eval::glob_match;
    assert!(glob_match("[", "["));
    assert!(glob_match("test[", "test["));
    assert!(glob_match("*", ""));

    // A/a 修饰符当前为原样返回
    env.set_scalar("REL", "../p");
    assert_eq!(expand_str("${REL:A}", &env, &opts()).unwrap(), "../p");
    assert_eq!(expand_str("${REL:a}", &env, &opts()).unwrap(), "../p");
}
