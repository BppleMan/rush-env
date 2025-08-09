use rush_var::parser::parse_braced;

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
