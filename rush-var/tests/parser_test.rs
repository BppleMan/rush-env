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

#[test]
fn replace_with_param_in_pat_and_repl() {
    let e = parse_braced("${var/$FOO/${BAR}}").unwrap();
    if let ParamExpr::Replace { pat, repl, .. } = e {
        assert!(matches!(pat.get(0), Some(Word::Param(_))));
        assert!(matches!(repl.get(0), Some(Word::Param(_))));
    } else {
        panic!("expected Replace");
    }
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
    } else {
        panic!("expected Defaulting");
    }
}

#[test]
fn word_until_unclosed_cmd_subst_errors() {
    assert!(parse_braced("${v:-$(echo}").is_err());
}

#[test]
fn index_negative_number_parses() {
    let e = parse_braced("${arr[-1]}").unwrap();
    if let ParamExpr::Ref { index, .. } = e {
        match index {
            Index::One(n) => assert_eq!(n, -1),
            _ => panic!("expected One(-1)"),
        }
    } else {
        panic!("expected Ref");
    }
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
        } else {
            panic!("expected inner Substring");
        }
    } else {
        panic!("expected outer Substring");
    }
}

#[test]
fn find_expansions_ignores_lonely_dollar() {
    let s = "foo $ bar";
    let exps = rush_var::parser::find_expansions(s).unwrap();
    assert!(exps.is_empty());
}

#[test]
fn replace_missing_repl_reports_error() {
    // ${var/pattern} 缺少第二个 '/'，应报错
    assert!(parse_braced("${var/pattern}").is_err());
}

#[test]
fn colon_invalid_operation_reports_error() {
    // 在冒号后跟随不支持的内容，触发错误路径
    assert!(parse_braced("${var:XYZ}").is_err());
}

#[test]
fn replace_missing_repl_variants_error() {
    // 缺少替换部分的其它 scope 变体
    assert!(parse_braced("${var//pattern}").is_err());
    assert!(parse_braced("${var/#pattern}").is_err());
    assert!(parse_braced("${var/%pattern}").is_err());
}

#[test]
fn defaulting_empty_word_and_colonless_variants() {
    // 空 word 情况
    let e = parse_braced("${v:?}").unwrap();
    if let rush_var::ast::ParamExpr::Defaulting { colon, .. } = e {
        assert!(colon);
    } else {
        panic!("expected Defaulting");
    }

    // 无冒号的四种操作，空 word 合法
    assert!(parse_braced("${v-}").is_ok());
    assert!(parse_braced("${v=}").is_ok());
    assert!(parse_braced("${v+}").is_ok());
    assert!(parse_braced("${v?}").is_ok());
}

#[test]
fn flags_missing_rparen_and_unknown_flag() {
    // 缺少右括号
    assert!(parse_braced("${(Uv}").is_err());
    assert!(parse_braced("${(U}").is_err());
    // 未知 flag
    assert!(parse_braced("${(X)var}").is_err());
}

#[test]
fn path_modifiers_multi_colons_and_empty_sequence() {
    // 多个冒号穿插的路径修饰符序列
    let e = parse_braced("${var:h::t::r:e}").unwrap();
    if let rush_var::ast::ParamExpr::Modifiers { mods, .. } = e {
        assert!(mods.contains(&rush_var::ast::PathMod::H));
        assert!(mods.contains(&rush_var::ast::PathMod::T));
        assert!(mods.contains(&rush_var::ast::PathMod::R));
        assert!(mods.contains(&rush_var::ast::PathMod::E));
    } else {
        panic!("expected Modifiers");
    }

    // 只有冒号时应解析为空修饰符序列（随后可能接其它操作或直接闭合）
    let e = parse_braced("${var:}");
    assert!(e.is_err()); // 当前实现对裸 ':' 报错
}

#[test]
fn remove_ops_with_empty_pattern_are_noop() {
    // 空 pattern，被 parse_word_until 截止于 '}'，应作为空串；语义上是 no-op
    let e = parse_braced("${v#}").unwrap();
    if let rush_var::ast::ParamExpr::Remove { .. } = e {
    } else {
        panic!("expected Remove");
    }
    let e = parse_braced("${v##}").unwrap();
    if let rush_var::ast::ParamExpr::Remove { .. } = e {
    } else {
        panic!("expected Remove");
    }
    let e = parse_braced("${v%}").unwrap();
    if let rush_var::ast::ParamExpr::Remove { .. } = e {
    } else {
        panic!("expected Remove");
    }
    let e = parse_braced("${v%%}").unwrap();
    if let rush_var::ast::ParamExpr::Remove { .. } = e {
    } else {
        panic!("expected Remove");
    }
}

#[test]
fn replace_then_path_modifiers_chain() {
    // ${X/old/new:h:t} -> 在当前实现中，repl 读到 '}'，因此 ":h:t" 属于 repl 文本
    let e = parse_braced("${X/old/new:h:t}").unwrap();
    if let ParamExpr::Replace { repl, .. } = e {
        assert_eq!(repl.len(), 1);
        match &repl[0] {
            Word::Text(t) => assert_eq!(t, "new:h:t"),
            other => panic!("unexpected repl: {:?}", other),
        }
    } else {
        panic!("expected Replace outer");
    }
}

#[test]
fn flags_then_replace_then_parse_ok() {
    // ${(U)v/old/new} -> Replace 外层，内层为 ZshFlags(U)
    let e = parse_braced("${(U)v/old/new}").unwrap();
    if let ParamExpr::Replace { inner, .. } = e {
        if let ParamExpr::ZshFlags { flags, .. } = inner.as_ref() {
            assert_eq!(flags.len(), 1);
            assert!(matches!(flags[0].kind, ZFlag::U));
        } else {
            panic!("inner should be ZshFlags(U)");
        }
    } else {
        panic!("expected Replace outer");
    }
}

#[test]
fn positional_two_digits_in_braced() {
    // ${12} -> 多位位置参数应被识别为 positional: Some(12)
    let e = parse_braced("${12}").unwrap();
    if let ParamExpr::Ref { target, .. } = e {
        assert_eq!(target.positional, Some(12));
        assert!(target.special.is_none());
        assert_eq!(target.name, "12");
    } else {
        panic!("expected Ref with positional 12");
    }
}

#[test]
fn defaulting_colonless_then_substring_chains() {
    // ${v-foo:1:2} -> 在当前实现中，colonless 默认值会将余下内容当作 word 直至 '}'
    let e = parse_braced("${v-foo:1:2}").unwrap();
    if let ParamExpr::Defaulting { colon, op, word, .. } = e {
        assert!(!colon);
        assert!(matches!(op, DefaultOp::Dash));
        assert_eq!(word.len(), 1);
        match &word[0] {
            Word::Text(t) => assert_eq!(t, "foo:1:2"),
            other => panic!("unexpected word: {:?}", other),
        }
    } else {
        panic!("expected Defaulting outer");
    }
}

#[test]
fn modifiers_then_defaulting_chain() {
    // ${v:h:-fb} -> 解析器先吃完 :h 与随后的 ':'，随后看到 '-' 走 colonless 默认值
    let e = parse_braced("${v:h:-fb}").unwrap();
    if let ParamExpr::Defaulting { inner, colon, op, word } = e {
        assert!(!colon);
        assert!(matches!(op, DefaultOp::Dash));
        assert_eq!(word.len(), 1);
        match &word[0] {
            Word::Text(t) => assert_eq!(t, "fb"),
            other => panic!("unexpected word: {:?}", other),
        }
        if let ParamExpr::Modifiers { mods, .. } = inner.as_ref() {
            assert_eq!(mods.as_slice(), [PathMod::H]);
        } else {
            panic!("inner should be Modifiers");
        }
    } else {
        panic!("expected Defaulting outer");
    }
}

#[test]
fn bang_indirection_without_name_errors() {
    // ${!} -> 有间接符但没有目标名称，应报错
    assert!(parse_braced("${!}").is_err());
}

#[test]
fn wordlist_simple_positional_like_kept_literal() {
    // 在 wordlist 中的 $1 不被解析为参数，按字面"$1"
    let e = parse_braced("${X:-$1}").unwrap();
    if let ParamExpr::Defaulting { word, .. } = e {
        assert_eq!(word.len(), 1);
        if let Word::Text(t) = &word[0] {
            assert_eq!(t, "$1");
        } else {
            panic!("expected Text('$1')");
        }
    } else {
        panic!("expected Defaulting");
    }
}

#[test]
fn find_expansions_variable_with_digits_and_underscore() {
    // $var1_2 应整体作为简单展开被定位
    let s = "$var1_2 and ${name1_2}";
    let exps = rush_var::parser::find_expansions(s).unwrap();
    // 预期有两个展开：$var1_2 与 ${name1_2}
    assert_eq!(exps.len(), 2);
}

#[test]
fn substring_negative_with_length() {
    // ${v:-5:3} -> 冒号后负偏移且包含长度
    let e = parse_braced("${v:-5:3}").unwrap();
    if let ParamExpr::Substring { offset, len, .. } = e {
        assert_eq!(offset, -5);
        assert_eq!(len, Some(3));
    } else {
        panic!("expected Substring");
    }
}

#[test]
fn length_then_indirection_chain() {
    // ${#!var} -> length 与间接组合，顺序为 Indirection 再 Length 包裹
    let e = parse_braced("${#!var}").unwrap();
    if let ParamExpr::Length { inner } = e {
        if let ParamExpr::Indirection { inner: ind_inner, style } = inner.as_ref() {
            assert!(matches!(style, IndirectStyle::BashBang));
            if let ParamExpr::Ref { target, .. } = ind_inner.as_ref() {
                assert_eq!(target.name, "var");
            } else {
                panic!("indirection inner should be Ref");
            }
        } else {
            panic!("expected Indirection inside Length");
        }
    } else {
        panic!("expected Length outer");
    }
}

#[test]
fn indirection_then_replace() {
    // ${!v/old/new} -> Replace 外层，内层为 Indirection
    let e = parse_braced("${!v/old/new}").unwrap();
    if let ParamExpr::Replace { inner, .. } = e {
        if let ParamExpr::Indirection { inner: ind_inner, .. } = inner.as_ref() {
            assert!(matches!(ind_inner.as_ref(), ParamExpr::Ref { .. }));
        } else {
            panic!("inner should be Indirection");
        }
    } else {
        panic!("expected Replace outer");
    }
}
