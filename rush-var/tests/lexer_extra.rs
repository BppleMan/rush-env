use rush_var::lexer::Lexer;

#[test]
fn read_flag_args_mixed_delims_with_escape_and_consecutive() {
    // 不同分隔符 + 转义 + 连续分隔
    let mut lx = Lexer::new("{a\\:b::c}");
    let args = lx.read_flag_args('{', '}');
    assert_eq!(args, vec!["a\\:b::c"]);

    // 相同分隔符，只有一个空参数
    let mut lx2 = Lexer::new(":");
    let args2 = lx2.read_flag_args(':', ':');
    assert_eq!(args2, vec![""]);
}
