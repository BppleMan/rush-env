use rush_var::ast::*;
use rush_var::parser::parse_braced;

#[test]
fn word_parsing_with_simple_var_in_default_word() {
    // ${UNSET:-$FOO-sfx} -> 词中包含简单 $var 和后续文本
    let expr = parse_braced("${UNSET:-$FOO-sfx}").unwrap();
    match expr {
        ParamExpr::Defaulting { word, .. } => {
            assert_eq!(word.len(), 2);
            match (&word[0], &word[1]) {
                (Word::Param(_), Word::Text(t)) => assert_eq!(t, "-sfx"),
                other => panic!("unexpected word parts: {:?}", other),
            }
        }
        other => panic!("expected Defaulting, got {:?}", other),
    }
}

#[test]
fn word_parsing_with_cmd_and_arith_subst() {
    // 有效的命令替换与算术替换都应被解析为 Word 片段
    let expr = parse_braced("${VAR:-$(echo hi)}").unwrap();
    match expr {
        ParamExpr::Defaulting { word, .. } => {
            assert!(matches!(word[0], Word::CmdSubst(_)));
        }
        _ => panic!("expected Defaulting with CmdSubst"),
    }

    let expr = parse_braced("${VAR:-$((1+2))}").unwrap();
    match expr {
        ParamExpr::Defaulting { word, .. } => {
            assert!(matches!(word[0], Word::ArithSubst(_)));
        }
        _ => panic!("expected Defaulting with ArithSubst"),
    }
}

#[test]
fn parse_flags_multiple_in_one_group() {
    // 多个旗标连写
    let expr = parse_braced("${(ULCqQfzuovtVP)var}").unwrap();
    match expr {
        ParamExpr::ZshFlags { flags, .. } => {
            assert!(flags.len() >= 8);
        }
        _ => panic!("expected ZshFlags"),
    }
}

#[test]
fn parse_modifiers_chain_h_e() {
    // 纯路径修饰符链 :h:e
    let expr = parse_braced("${name:h:e}").unwrap();
    match expr {
        ParamExpr::Modifiers { mods, .. } => {
            assert!(mods.contains(&PathMod::H));
            assert!(mods.contains(&PathMod::E));
        }
        other => panic!("unexpected top-level expr: {:?}", other),
    }
}

#[test]
fn parse_target_selected_specials() {
    // 这里排除 ${!}，因为 '!' 在本实现中被用作间接展开前缀
    for s in ["${*}", "${@}", "${-}", "${$}"] {
        let expr = parse_braced(s).unwrap();
        match expr {
            ParamExpr::Ref { target, .. } => {
                assert!(target.special.is_some(), "{} should be special", s);
            }
            _ => panic!("expected Ref for {}", s),
        }
    }
}

#[test]
fn defaulting_qmark_without_word_is_allowed() {
    // ${var:?} -> 空 word，命中 ':' 分支下的 '?' 处理路径
    let expr = parse_braced("${var:?}").unwrap();
    match expr {
        ParamExpr::Defaulting { op, colon, word, .. } => {
            assert!(matches!(op, DefaultOp::QMark));
            assert!(colon);
            assert!(word.is_empty());
        }
        _ => panic!("expected Defaulting with QMark"),
    }
}
