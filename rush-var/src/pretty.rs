use crate::{Token, TokenKind, TokenStream};

/// 把 token 流转成稳定、可读、可比对的文本（适合 insta 快照）
/// 约定：
/// - 对“带字面值”的 token（Text/Ident/SpecialParam/SignedInt/FlagChar/SepString/Pattern）
///   打印切片内容（经转义）
/// - 其它符号类仅打印种类名与 span
pub fn pretty_tokens(stream: &TokenStream) -> String {
    fn escape_literal(literal: &str) -> String {
        let mut escaped = String::with_capacity(literal.len());
        for ch in literal.chars() {
            match ch {
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\\""),
                _ => escaped.push(ch),
            }
        }
        escaped
    }

    fn render_line(stream: &TokenStream, token: &Token) -> String {
        use TokenKind::*;
        let span_text = format!("@{}..{}", token.span.start, token.span.end);

        match token.kind {
            Text | Ident | SpecialParam | SignedInt | FlagChar | SepString | Pattern => {
                let slice = stream.text(token);
                let escaped = escape_literal(slice);
                format!("{:12} {:<} \"{}\"\n", kind_name(&token.kind), span_text, escaped)
            }
            _ => format!("{:12} {}\n", kind_name(&token.kind), span_text),
        }
    }

    fn kind_name(kind: &TokenKind) -> &'static str {
        use TokenKind::*;
        match kind {
            // 带字面值
            Text => "Text",
            Ident => "Ident",
            SpecialParam => "SpecialParam",
            SignedInt => "SignedInt",
            FlagChar => "FlagChar",
            SepString => "SepString",
            Pattern => "Pattern",

            // 结构/分隔
            Dollar => "Dollar",
            LBrace => "LBrace",
            RBrace => "RBrace",
            LParen => "LParen",
            RParen => "RParen",
            LBracket => "LBracket",
            RBracket => "RBracket",
            Colon => "Colon",
            Comma => "Comma",

            // 双/单字符操作符
            DoubleHash => "DoubleHash",
            Hash => "Hash",
            DoublePercent => "DoublePercent",
            Percent => "Percent",
            DoubleSlash => "DoubleSlash",
            Slash => "Slash",
            Plus => "Plus",
            Minus => "Minus",
            Question => "Question",
            Equals => "Equals",

            // 兜底
            Unknown => "Unknown",
        }
    }

    let mut buffer = String::new();
    for token in &stream.tokens {
        buffer.push_str(&render_line(stream, token));
    }
    buffer
}
