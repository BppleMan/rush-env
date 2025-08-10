use rush_var::{Source, Span, Token, TokenKind, TokenStream, pretty_tokens};

#[test]
fn pretty_escapes_and_symbols() {
    let src = Source { src: "a\n\t\\\"b" };
    let tokens = vec![
        Token::new(TokenKind::Text, Span::new(0, 6)),   // 包含 \n \t \\ \"
        Token::new(TokenKind::Dollar, Span::new(6, 7)), // 无字面值分支
        Token::new(TokenKind::Plus, Span::new(6, 7)),   // 再走一次“仅名字+span”路径
    ];
    let ts = TokenStream { source: src, tokens };
    let dump = pretty_tokens(&ts);
    assert!(dump.contains("\\n"));
    assert!(dump.contains("\\t"));
    assert!(dump.contains("\\\\"));
    assert!(dump.contains("\\\""));
    assert!(dump.contains("Dollar"));
    assert!(dump.contains("Plus"));
}
