use rush_var::ast::*;
use rush_var::parser::parse_braced;

#[test]
fn replace_with_param_in_pat_and_repl() {
    let e = parse_braced("${var/$FOO/${BAR}}").unwrap();
    if let ParamExpr::Replace { pat, repl, .. } = e {
        assert!(matches!(pat.get(0), Some(Word::Param(_))));
        assert!(matches!(repl.get(0), Some(Word::Param(_))));
    } else { panic!("expected Replace"); }
}

#[test]
fn default_word_with_mixed_nested_kinds() {
    let e = parse_braced("${v:-pre${X}$(echo y)$((1+2))post}").unwrap();
    if let ParamExpr::Defaulting { word, .. } = e {
        assert!(matches!(word.get(0), Some(Word::Text(t)) if t == "pre"));
        assert!(matches!(word.get(1), Some(Word::Param(_))));
        assert!(matches!(word.get(2), Some(Word::CmdSubst(s)) if s.contains("echo")));
        assert!(matches!(word.get(3), Some(Word::ArithSubst(s)) if s.contains("1+2")));
        assert!(matches!(word.get(4), Some(Word::Text(t)) if t == "post"));
    } else { panic!("expected Defaulting"); }
}

#[test]
fn word_until_unclosed_cmd_subst_errors() {
    assert!(parse_braced("${v:-$(echo}").is_err());
}

#[test]
fn index_negative_number_parses() {
    let e = parse_braced("${arr[-1]}").unwrap();
    if let ParamExpr::Ref { index, .. } = e {
        match index { Index::One(n) => assert_eq!(n, -1), _ => panic!("expected One(-1)") }
    } else { panic!("expected Ref"); }
}

#[test]
fn substring_twice_chains_as_nested() {
    let e = parse_braced("${v:1:2:3}").unwrap();
    if let ParamExpr::Substring { inner, offset, len } = e {
        // 第二次 :3 是外层
        assert_eq!(offset, 3);
        assert_eq!(len, None);
        if let ParamExpr::Substring { offset: o2, len: l2, .. } = inner.as_ref() {
            assert_eq!(*o2, 1);
            assert_eq!(*l2, Some(2));
        } else { panic!("expected inner Substring"); }
    } else { panic!("expected outer Substring"); }
}

#[test]
fn find_expansions_ignores_lonely_dollar() {
    let s = "foo $ bar";
    let exps = rush_var::parser::find_expansions(s).unwrap();
    assert!(exps.is_empty());
}
