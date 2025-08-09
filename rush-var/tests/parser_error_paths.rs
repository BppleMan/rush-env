use rush_var::parser::parse_braced;

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
