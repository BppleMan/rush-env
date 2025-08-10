use crate::{Span, Token, TokenKind};

#[derive(Clone, Copy)]
pub struct Cursor<'a> {
    pub bytes: &'a [u8],
    pub i: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { bytes: s.as_bytes(), i: 0 }
    }

    #[inline]
    pub fn eof(&self) -> bool {
        self.i >= self.bytes.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.i).copied()
    }

    #[inline]
    pub fn peek2(&self) -> Option<(u8, u8)> {
        if self.i + 1 < self.bytes.len() {
            Some((self.bytes[self.i], self.bytes[self.i + 1]))
        } else {
            None
        }
    }

    #[inline]
    pub fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.i += 1;
        Some(b)
    }

    #[inline]
    pub fn starts_with(&self, a: u8, b: u8) -> bool {
        self.i + 1 < self.bytes.len() && self.bytes[self.i] == a && self.bytes[self.i + 1] == b
    }
}

#[inline]
pub fn is_ident_head(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

#[inline]
pub fn is_ident_tail(b: u8) -> bool {
    is_ident_head(b) || b.is_ascii_digit()
}

pub fn push_text_if_any(start: &mut Option<usize>, cur: &Cursor, out: &mut Vec<Token>) {
    if let Some(s) = start.take() {
        if cur.i > s {
            out.push(Token::new(TokenKind::Text, Span::new(s, cur.i)));
        }
    }
}
