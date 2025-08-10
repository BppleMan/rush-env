use crate::Span;
use std::num::ParseIntError;

#[derive(Clone, Copy)]
pub struct Source<'a> {
    pub src: &'a str,
}

impl<'a> Source<'a> {
    #[inline]
    pub fn slice(&self, sp: Span) -> &'a str {
        &self.src[sp.start as usize..sp.end as usize]
    }
    #[inline]
    pub fn first_char(&self, sp: Span) -> Option<char> {
        self.slice(sp).chars().next()
    }
    #[inline]
    pub fn parse_i64(&self, sp: Span) -> Result<i64, ParseIntError> {
        self.slice(sp).parse::<i64>()
    }
}

pub struct TokenStream<'a> {
    pub source: Source<'a>,
    pub tokens: Vec<crate::token::Token>,
}

impl<'a> TokenStream<'a> {
    #[inline]
    pub fn text(&self, t: &crate::token::Token) -> &'a str {
        self.source.slice(t.span)
    }
}
