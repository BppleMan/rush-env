use crate::ast::{Error, ParamExpr};

/// Standalone helper to parse a single braced parameter expansion string like "${...}".
/// This delegates to the main `Parser` implementation for the actual work.
pub fn parse_braced(source: &str) -> Result<ParamExpr, Error> {
    let mut parser = super::Parser::new(source);
    parser.parse_braced()
}
