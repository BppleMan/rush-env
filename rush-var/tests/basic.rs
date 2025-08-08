use rush_var::{expand_str, Env, Options};

#[test]
fn test_basic_and_length() {
    let mut env = Env::new();
    env.set_scalar("FOO", "bar");
    let opts = Options::default();
    let out = expand_str("$FOO/${#FOO}", &env, &opts).unwrap();
    assert_eq!(out, "bar/3");
}

#[test]
fn test_defaulting() {
    let env = Env::new();
    let opts = Options::default();
    let out = expand_str("${A:-x}", &env, &opts).unwrap();
    assert_eq!(out, "x");
    let mut env2 = Env::new();
    env2.set_scalar("A", "");
    let out2 = expand_str("${A:=y}", &env2, &opts).unwrap();
    assert_eq!(out2, "y");
}

#[test]
fn test_remove_replace() {
    let mut env = Env::new();
    env.set_scalar("PATH", "/usr/bin" );
    env.set_scalar("WORD", "helloworld" );
    let opts = Options::default();
    assert_eq!(expand_str("${PATH#/usr}", &env, &opts).unwrap(), "/bin");
    assert_eq!(expand_str("${WORD%world}", &env, &opts).unwrap(), "hello");
    assert_eq!(expand_str("${WORD/hello/hi}", &env, &opts).unwrap(), "hiworld");
    assert_eq!(expand_str("${WORD//l/_}", &env, &opts).unwrap(), "he__owor_d");
}

#[test]
fn test_substring_indirect() {
    let mut env = Env::new();
    env.set_scalar("NAME", "abcdef");
    env.set_scalar("A", "NAME");
    let opts = Options::default();
    assert_eq!(expand_str("${NAME:2:3}", &env, &opts).unwrap(), "cde");
    assert_eq!(expand_str("${!A}", &env, &opts).unwrap(), "abcdef");
}
