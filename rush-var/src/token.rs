use crate::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // 文本/名字/常量（字面值用 span 到 Source 切）
    Text,         // 普通文本片段（外层/word 内聚合）
    Ident,        // [A-Za-z_][A-Za-z0-9_]*
    SpecialParam, // *, @, #, ?, -, $, !, _, 0..9
    SignedInt,    // [-+]?[0-9]+

    // 结构分隔
    Dollar,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,

    // 双字符优先
    DoubleHash,
    Hash,
    DoublePercent,
    Percent,
    DoubleSlash,
    Slash,

    // flags 内原子
    FlagChar,  // 单字母 flag（s/j/q/Q/u/l/h/t/r/e/...）
    SepString, // (s:SEP:) / (j:SEP:) 的 SEP（不含两侧冒号）

    // 模式字面
    Pattern,

    // 特殊符号
    Plus,
    Minus,
    Question,
    Equals,
    Comma,

    // 兜底
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

macro_rules! token_ctor {
    ($name:ident, $kind:expr) => {
        #[inline]
        pub fn $name(span: Span) -> Self {
            Self::new($kind, span)
        }
    };
}

impl Token {
    #[inline]
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    token_ctor!(text, TokenKind::Text);
    token_ctor!(ident, TokenKind::Ident);
    token_ctor!(special_param, TokenKind::SpecialParam);
    token_ctor!(signed_int, TokenKind::SignedInt);

    token_ctor!(dollar, TokenKind::Dollar);
    token_ctor!(lbrace, TokenKind::LBrace);
    token_ctor!(rbrace, TokenKind::RBrace);
    token_ctor!(lparen, TokenKind::LParen);
    token_ctor!(rparen, TokenKind::RParen);
    token_ctor!(lbracket, TokenKind::LBracket);
    token_ctor!(rbracket, TokenKind::RBracket);
    token_ctor!(colon, TokenKind::Colon);

    token_ctor!(double_hash, TokenKind::DoubleHash);
    token_ctor!(hash, TokenKind::Hash);
    token_ctor!(double_percent, TokenKind::DoublePercent);
    token_ctor!(percent, TokenKind::Percent);
    token_ctor!(double_slash, TokenKind::DoubleSlash);
    token_ctor!(slash, TokenKind::Slash);

    token_ctor!(flag_char, TokenKind::FlagChar);
    token_ctor!(sep_string, TokenKind::SepString);

    token_ctor!(pattern, TokenKind::Pattern);

    token_ctor!(plus, TokenKind::Plus);
    token_ctor!(minus, TokenKind::Minus);
    token_ctor!(question, TokenKind::Question);
    token_ctor!(equals, TokenKind::Equals);
    token_ctor!(comma, TokenKind::Comma);

    token_ctor!(unknown, TokenKind::Unknown);
}
