use rush_var::{
    Env, Options,
    ast::{Index, ParamExpr, ParamExpr::Ref, Target},
    eval::evaluate_expr,
    expand_str,
};

fn opts() -> Options {
    Options::default()
}

#[test]
fn index_strslice_and_overflow_paths() {
    let mut env = Env::new();
    env.set_scalar("FOO", "bar");
    env.set_array("ARR", vec!["x", "y", "z"]);

    // StrSlice on scalar via direct AST
    let expr = ParamExpr::Ref {
        target: Target {
            name: "FOO".into(),
            special: None,
            positional: None,
        },
        index: Index::StrSlice(0, 2),
    };
    assert_eq!(evaluate_expr(&expr, &env, &opts()).unwrap(), "ba");

    let expr = ParamExpr::Ref {
        target: Target {
            name: "FOO".into(),
            special: None,
            positional: None,
        },
        index: Index::StrSlice(10, 2),
    };
    assert_eq!(evaluate_expr(&expr, &env, &opts()).unwrap(), "");

    // Overflow index on array should error
    let expr = ParamExpr::Ref {
        target: Target {
            name: "ARR".into(),
            special: None,
            positional: None,
        },
        index: Index::One(i64::MAX),
    };
    assert!(evaluate_expr(&expr, &env, &opts()).is_err());

    // Zero index returns empty (out-of-range handling)
    let expr = ParamExpr::Ref {
        target: Target {
            name: "ARR".into(),
            special: None,
            positional: None,
        },
        index: Index::One(0),
    };
    assert_eq!(evaluate_expr(&expr, &env, &opts()).unwrap(), "");
}

#[test]
fn undefined_special_parameter_errors() {
    let env = Env::new();
    // Build a Ref to an unknown special parameter
    let expr = ParamExpr::Ref {
        target: Target {
            name: "^".into(),
            special: Some('^'),
            positional: None,
        },
        index: Index::None,
    };
    assert!(evaluate_expr(&expr, &env, &opts()).is_err());
}

#[test]
fn cmd_and_arith_subst_in_word_list_are_literal_when_exec_disabled() {
    let mut env = Env::new();
    // Unset X so defaulting path is taken
    let out = expand_str("${X:-$(echo hi)}", &env, &opts()).unwrap();
    assert_eq!(out, "$(echo hi)");
    let out = expand_str("${X:-$((1+2))}", &env, &opts()).unwrap();
    assert_eq!(out, "$((1+2))");
}

#[test]
fn vdisplay_unique_sort_and_split_s_flags() {
    let mut env = Env::new();
    env.set_scalar("S", "a\nb\t\\");
    assert_eq!(expand_str("${(V)S}", &env, &opts()).unwrap(), "a\\nb\\t\\\\");

    env.set_scalar("W", "b a b");
    assert_eq!(expand_str("${(u)W}", &env, &opts()).unwrap(), "b a");
    assert_eq!(expand_str("${(o)W}", &env, &opts()).unwrap(), "a b b");
    assert_eq!(expand_str("${(O)W}", &env, &opts()).unwrap(), "b b a");

    env.set_scalar("STR", "a|b|c");
    assert_eq!(expand_str("${(s:|:)STR}", &env, &opts()).unwrap(), "a b c");
}

#[test]
fn path_modifier_t_on_dir_trims_to_empty_basename() {
    let mut env = Env::new();
    env.set_scalar("P", "/usr/local/bin/");
    assert_eq!(expand_str("${P:t}", &env, &opts()).unwrap(), "");
}

#[test]
fn defaulting_qmark_raises_eval_error_with_message() {
    let env = Env::new();
    let err = expand_str("${UNSET:?oops}", &env, &opts()).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("oops"));
}
