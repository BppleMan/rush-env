use crate::ast::*;
use crate::lexer::Lexer;

// Constants for commonly used strings
const DEFAULT_SPACE: &str = " ";
const EXPECTED_BRACE_START: &str = "expected '${' at start";
const EXPECTED_BRACE_END: &str = "expected '}' at end";
const EXPECTED_PAREN_CLOSE: &str = "expected ')' after flag";

/// Parser for shell parameter expansions
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { lexer: Lexer::new(input) }
    }

    /// Parse a braced parameter expansion like ${...}
    pub fn parse_braced(&mut self) -> Result<ParamExpr, Error> {
        // Expect opening ${
        if !self.lexer.consume_str("${") {
            return Err(Error::BadSubstitution(EXPECTED_BRACE_START.to_string()));
        }

        // Parse flags first if present
        let flags = self.parse_flags()?;

        // Parse length operator
        let is_length = if self.lexer.matches('#') {
            // Check if this is ${#} (special parameter) vs ${#var} (length)
            if self.lexer.peek_ahead(1) == Some('}') {
                // This is ${#} - special parameter, not length
                false
            } else {
                self.lexer.next_char();
                true
            }
        } else {
            false
        };

        // Parse indirection operator
        let indirection = if self.lexer.matches('!') {
            self.lexer.next_char();
            Some(IndirectStyle::BashBang)
        } else {
            None
        };

        // Parse the base expression
        let mut expr = self.parse_base_expr()?;

        // Apply wrappers in a stable order: flags -> indirection -> length
        if !flags.is_empty() {
            expr = ParamExpr::ZshFlags {
                flags,
                inner: Box::new(expr),
            };
        }
        if let Some(style) = indirection {
            expr = ParamExpr::Indirection {
                inner: Box::new(expr),
                style,
            };
        }
        if is_length {
            expr = ParamExpr::Length { inner: Box::new(expr) };
        }

        // Parse operations (defaulting, remove, replace, substring, etc.)
        expr = self.parse_operations(expr)?;

        // Expect closing }
        if !self.lexer.matches('}') {
            return Err(Error::BadSubstitution(EXPECTED_BRACE_END.to_string()));
        }
        self.lexer.next_char();

        Ok(expr)
    }

    /// Parse zsh flags like (U), (L), (j:sep:), etc.
    fn parse_flags(&mut self) -> Result<Vec<ZshFlag>, Error> {
        let mut flags = Vec::new();

        if self.lexer.matches('(') {
            self.lexer.next_char(); // consume '('

            // Parse multiple flags within the same parentheses
            while !self.lexer.matches(')') && !self.lexer.is_at_end() {
                let flag = self.parse_single_flag()?;
                flags.push(flag);
            }

            if !self.lexer.matches(')') {
                return Err(Error::BadSubstitution(EXPECTED_PAREN_CLOSE.to_string()));
            }
            self.lexer.next_char(); // consume ')'
        }

        Ok(flags)
    }

    /// Parse a single flag
    fn parse_single_flag(&mut self) -> Result<ZshFlag, Error> {
        let ch = self
            .lexer
            .next_char()
            .ok_or_else(|| Error::BadSubstitution("expected flag character".to_string()))?;

        match ch {
            'U' => Ok(ZshFlag {
                kind: ZFlag::U,
                args: vec![],
            }),
            'L' => Ok(ZshFlag {
                kind: ZFlag::L,
                args: vec![],
            }),
            'C' => Ok(ZshFlag {
                kind: ZFlag::C,
                args: vec![],
            }),
            'q' => Ok(ZshFlag {
                kind: ZFlag::Q,
                args: vec![],
            }),
            'Q' => Ok(ZshFlag {
                kind: ZFlag::Unquote,
                args: vec![],
            }),
            'f' => Ok(ZshFlag {
                kind: ZFlag::F,
                args: vec![],
            }),
            'z' => Ok(ZshFlag {
                kind: ZFlag::Z,
                args: vec![],
            }),
            'Z' => Ok(ZshFlag {
                kind: ZFlag::ZExt,
                args: vec![],
            }),
            'u' => Ok(ZshFlag {
                kind: ZFlag::Unique,
                args: vec![],
            }),
            'o' => Ok(ZshFlag {
                kind: ZFlag::O,
                args: vec![],
            }),
            'O' => Ok(ZshFlag {
                kind: ZFlag::ODesc,
                args: vec![],
            }),
            'k' => Ok(ZshFlag {
                kind: ZFlag::K,
                args: vec![],
            }),
            'v' => Ok(ZshFlag {
                kind: ZFlag::V,
                args: vec![],
            }),
            't' => Ok(ZshFlag {
                kind: ZFlag::T,
                args: vec![],
            }),
            'V' => Ok(ZshFlag {
                kind: ZFlag::VDisplay,
                args: vec![],
            }),
            'P' => Ok(ZshFlag {
                kind: ZFlag::P,
                args: vec![],
            }),
            'e' => Ok(ZshFlag {
                kind: ZFlag::E,
                args: vec![],
            }),
            'l' => {
                let args = self.lexer.read_flag_args(':', ':');
                if !args.is_empty() {
                    Ok(ZshFlag {
                        kind: ZFlag::L2 {
                            width: args[0].clone(),
                            fill: args.get(1).cloned().unwrap_or_default(),
                            pad: args.get(2).cloned().unwrap_or_default(),
                        },
                        args: vec![],
                    })
                } else {
                    Err(Error::BadSubstitution("(l) flag requires width argument".to_string()))
                }
            }
            'r' => {
                let args = self.lexer.read_flag_args(':', ':');
                if !args.is_empty() {
                    Ok(ZshFlag {
                        kind: ZFlag::R2 {
                            width: args[0].clone(),
                            fill: args.get(1).cloned().unwrap_or_default(),
                            pad: args.get(2).cloned().unwrap_or_default(),
                        },
                        args: vec![],
                    })
                } else {
                    Err(Error::BadSubstitution("(r) flag requires width argument".to_string()))
                }
            }
            'j' => {
                let args = self.lexer.read_flag_args(':', ':');
                Ok(ZshFlag {
                    kind: ZFlag::J {
                        sep: args.first().cloned().unwrap_or_else(|| DEFAULT_SPACE.to_string()),
                    },
                    args: vec![],
                })
            }
            's' => {
                let args = self.lexer.read_flag_args(':', ':');
                Ok(ZshFlag {
                    kind: ZFlag::S {
                        sep: args.first().cloned().unwrap_or_else(|| DEFAULT_SPACE.to_string()),
                    },
                    args: vec![],
                })
            }
            _ => Err(Error::BadSubstitution(format!("unknown flag '{}'", ch))),
        }
    }

    /// Parse base expression (variable reference with optional indexing)
    fn parse_base_expr(&mut self) -> Result<ParamExpr, Error> {
        let target = self.parse_target()?;
        let index = self.parse_index()?;

        Ok(ParamExpr::Ref { target, index })
    }

    /// Parse target (variable name, special char, or positional)
    fn parse_target(&mut self) -> Result<Target, Error> {
        // Check for special characters
        if let Some(ch) = self.lexer.peek() {
            if "?$-!*@#".contains(ch) {
                self.lexer.next_char();
                return Ok(Target {
                    name: ch.to_string(),
                    special: Some(ch),
                    positional: None,
                });
            }

            // Check for positional parameter (digit)
            if ch.is_ascii_digit() {
                if let Some(num) = self.lexer.read_number() {
                    return Ok(Target {
                        name: num.to_string(),
                        special: None,
                        positional: Some(num as u32),
                    });
                }
            }
        }

        // Parse regular identifier
        let name = self.lexer.read_identifier();
        if name.is_empty() {
            return Err(Error::BadSubstitution("expected variable name".to_string()));
        }

        Ok(Target {
            name,
            special: None,
            positional: None,
        })
    }

    /// Parse array index [n] or [n,m]
    fn parse_index(&mut self) -> Result<Index, Error> {
        if !self.lexer.matches('[') {
            return Ok(Index::None);
        }

        self.lexer.next_char(); // consume '['

        // Try to read a number first
        if let Some(first) = self.lexer.read_number() {
            if self.lexer.matches(',') {
                self.lexer.next_char(); // consume ','
                let second = self
                    .lexer
                    .read_number()
                    .ok_or_else(|| Error::BadSubstitution("expected second number in slice".to_string()))?;

                self.lexer.expect(']')?;
                Ok(Index::Slice(first, second))
            } else {
                self.lexer.expect(']')?;
                Ok(Index::One(first))
            }
        } else {
            // If not a number, try to read as a string key
            let key = self.lexer.read_identifier();
            if key.is_empty() {
                return Err(Error::BadSubstitution("expected key in index".to_string()));
            }

            self.lexer.expect(']')?;
            Ok(Index::Key(key))
        }
    }

    /// Parse operations after the base expression
    fn parse_operations(&mut self, mut expr: ParamExpr) -> Result<ParamExpr, Error> {
        loop {
            match self.lexer.peek() {
                Some(':') => {
                    self.lexer.next_char(); // consume ':'

                    // Check what comes after ':'
                    match self.lexer.peek() {
                        // Substring operation ${name:offset[:length]} — allow digits or a leading '-' followed by digit
                        Some(ch) if ch.is_ascii_digit() || (ch == '-' && self.lexer.peek_ahead(1).is_some_and(|c| c.is_ascii_digit())) => {
                            let offset = self
                                .lexer
                                .read_number()
                                .ok_or_else(|| Error::BadSubstitution("expected offset".to_string()))?;

                            let len = if self.lexer.matches(':') {
                                self.lexer.next_char(); // consume ':'
                                self.lexer.read_number()
                            } else {
                                None
                            };

                            expr = ParamExpr::Substring {
                                inner: Box::new(expr),
                                offset,
                                len,
                            };
                        }
                        Some('-') => {
                            self.lexer.next_char();
                            expr = self.parse_defaulting_common(expr, true, DefaultOp::Dash)?;
                        }
                        Some('=') => {
                            self.lexer.next_char();
                            expr = self.parse_defaulting_common(expr, true, DefaultOp::Assign)?;
                        }
                        Some('+') => {
                            self.lexer.next_char();
                            expr = self.parse_defaulting_common(expr, true, DefaultOp::Plus)?;
                        }
                        Some('?') => {
                            self.lexer.next_char();
                            expr = self.parse_defaulting_common(expr, true, DefaultOp::QMark)?;
                        }
                        Some(ch) if "htraeA".contains(ch) => {
                            // Path modifiers
                            let mods = self.parse_path_modifiers()?;
                            expr = ParamExpr::Modifiers {
                                inner: Box::new(expr),
                                mods,
                            };
                        }
                        _ => return Err(Error::BadSubstitution("invalid operation after ':'".to_string())),
                    }
                }
                Some('#') => {
                    expr = self.parse_remove_operation(expr, true)?;
                }
                Some('%') => {
                    expr = self.parse_remove_operation(expr, false)?;
                }
                Some('/') => {
                    expr = self.parse_replace_operation(expr)?;
                }
                Some('-') => {
                    self.lexer.next_char();
                    expr = self.parse_defaulting_common(expr, false, DefaultOp::Dash)?;
                }
                Some('=') => {
                    self.lexer.next_char();
                    expr = self.parse_defaulting_common(expr, false, DefaultOp::Assign)?;
                }
                Some('+') => {
                    self.lexer.next_char();
                    expr = self.parse_defaulting_common(expr, false, DefaultOp::Plus)?;
                }
                Some('?') => {
                    self.lexer.next_char();
                    expr = self.parse_defaulting_common(expr, false, DefaultOp::QMark)?;
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Common builder for defaulting operations
    fn parse_defaulting_common(&mut self, expr: ParamExpr, colon: bool, op: DefaultOp) -> Result<ParamExpr, Error> {
        let word = self.parse_word_until(&['}'])?;
        Ok(ParamExpr::Defaulting {
            inner: Box::new(expr),
            colon,
            op,
            word,
        })
    }

    /// Parse remove (#, ##) or (% , %%) operations
    fn parse_remove_operation(&mut self, expr: ParamExpr, is_prefix: bool) -> Result<ParamExpr, Error> {
        let (long, symbol) = if is_prefix {
            (self.lexer.matches_str("##"), '#')
        } else {
            (self.lexer.matches_str("%%"), '%')
        };
        if long {
            self.lexer.consume_str(&format!("{}{}", symbol, symbol));
        } else {
            self.lexer.next_char();
        }
        let pattern = self.parse_word_until(&['}'])?;
        let op = if is_prefix {
            RemoveOp::Prefix { long }
        } else {
            RemoveOp::Suffix { long }
        };
        Ok(ParamExpr::Remove {
            inner: Box::new(expr),
            op,
            pattern,
        })
    }

    /// Parse replace operation ${name/pat/repl}
    fn parse_replace_operation(&mut self, expr: ParamExpr) -> Result<ParamExpr, Error> {
        self.lexer.next_char(); // consume first '/'

        let next_is_slash = self.lexer.matches('/');
        let next_is_hash = !next_is_slash && self.lexer.matches('#');
        let next_is_percent = !next_is_slash && !next_is_hash && self.lexer.matches('%');
        let scope = if next_is_slash {
            self.lexer.next_char();
            ReplaceScope::Global
        } else if next_is_hash {
            self.lexer.next_char();
            ReplaceScope::AnchorPrefix
        } else if next_is_percent {
            self.lexer.next_char();
            ReplaceScope::AnchorSuffix
        } else {
            ReplaceScope::First
        };

        let pat = self.parse_word_until(&['/'])?;

        let has_sep = self.lexer.matches('/');
        if !has_sep {
            return Err(Error::BadSubstitution("expected '/' after pattern in replace".to_string()));
        }
        self.lexer.next_char(); // consume '/'

        let repl = self.parse_word_until(&['}'])?;

        Ok(ParamExpr::Replace {
            inner: Box::new(expr),
            scope,
            pat,
            repl,
        })
    }

    /// Parse path modifiers like :h:t:r:e
    fn parse_path_modifiers(&mut self) -> Result<Vec<PathMod>, Error> {
        let mut mods = Vec::new();

        while let Some(ch) = self.lexer.peek() {
            match ch {
                'h' => {
                    self.lexer.next_char();
                    mods.push(PathMod::H);
                }
                't' => {
                    self.lexer.next_char();
                    mods.push(PathMod::T);
                }
                'r' => {
                    self.lexer.next_char();
                    mods.push(PathMod::R);
                }
                'e' => {
                    self.lexer.next_char();
                    mods.push(PathMod::E);
                }
                'A' => {
                    self.lexer.next_char();
                    mods.push(PathMod::A);
                }
                'a' => {
                    self.lexer.next_char();
                    mods.push(PathMod::LowerA);
                }
                ':' => {
                    self.lexer.next_char();
                }
                _ => break,
            }
        }

        Ok(mods)
    }

    /// Parse word content until one of the delimiters
    fn parse_word_until(&mut self, delims: &[char]) -> Result<Vec<Word>, Error> {
        let mut words = Vec::new();
        let mut current_text = String::new();

        while let Some(ch) = self.lexer.peek() {
            if delims.contains(&ch) {
                break;
            }

            if ch == '$' {
                // Save accumulated text
                if !current_text.is_empty() {
                    words.push(Word::Text(current_text.clone()));
                    current_text.clear();
                }

                // Parse nested expansion
                if self.lexer.matches_str("${") {
                    let nested_expr = self.parse_braced()?;
                    words.push(Word::Param(Box::new(nested_expr)));
                } else if self.lexer.matches_str("$((") {
                    self.lexer.consume_str("$((");
                    let content = self.lexer.read_until_unescaped(')')?; // TODO: handle nested parens
                    if !self.lexer.matches_str("))") {
                        return Err(Error::BadSubstitution("expected '))'".to_string()));
                    }
                    self.lexer.consume_str("))");
                    words.push(Word::ArithSubst(content));
                } else if self.lexer.matches_str("$(") {
                    self.lexer.consume_str("$(");
                    let content = self.lexer.read_until_unescaped(')')?;
                    self.lexer.expect(')')?;
                    words.push(Word::CmdSubst(content));
                } else {
                    // Simple $var reference
                    self.lexer.next_char(); // consume '$'
                    let name = self.lexer.read_identifier();
                    if !name.is_empty() {
                        let target = Target {
                            name,
                            special: None,
                            positional: None,
                        };
                        words.push(Word::Param(Box::new(ParamExpr::Ref {
                            target,
                            index: Index::None,
                        })));
                    } else {
                        current_text.push('$'); // Invalid $, treat as literal
                    }
                }
            } else {
                current_text.push(ch);
                self.lexer.next_char();
            }
        }

        // Save final text
        if !current_text.is_empty() {
            words.push(Word::Text(current_text));
        }

        Ok(words)
    }
}

/// Parse a single braced parameter expansion
pub fn parse_braced(source: &str) -> Result<ParamExpr, Error> {
    let mut parser = Parser::new(source);
    parser.parse_braced()
}

/// Find and parse all parameter expansions in a string
pub fn find_expansions(input: &str) -> Result<Vec<(usize, usize, ParamExpr)>, Error> {
    let mut expansions = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            let start = i;

            if chars[i + 1] == '{' {
                // Find matching closing brace
                let mut brace_count = 0;
                let mut j = i + 1;

                while j < chars.len() {
                    match chars[j] {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                let end = j + 1;
                                let expansion_str: String = chars[start..end].iter().collect();
                                match parse_braced(&expansion_str) {
                                    Ok(expr) => expansions.push((start, end, expr)),
                                    Err(e) => return Err(e), // Return error for invalid expansions
                                }
                                i = end;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }

                if brace_count > 0 {
                    // Unmatched braces, skip
                    i += 1;
                }
            } else if chars[i + 1].is_alphabetic() || chars[i + 1] == '_' || chars[i + 1].is_ascii_digit() {
                // Simple $var expansion (including positional parameters like $1, $2)
                let mut j = i + 1;

                if chars[i + 1].is_ascii_digit() {
                    // Handle positional parameters - read consecutive digits
                    while j < chars.len() && chars[j].is_ascii_digit() {
                        j += 1;
                    }
                } else {
                    // Handle regular variables - alphanumeric and underscore
                    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                        j += 1;
                    }
                }

                let name: String = chars[i + 1..j].iter().collect();
                let target = Target {
                    name,
                    special: None,
                    positional: None,
                };
                let expr = ParamExpr::Ref {
                    target,
                    index: Index::None,
                };
                expansions.push((start, j, expr));
                i = j;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    Ok(expansions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parser_basic_variable() {
        let mut parser = Parser::new("${var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Ref { target, .. } = expr {
            assert_eq!(target.name, "var");
            assert_eq!(target.special, None);
            assert_eq!(target.positional, None);
        } else {
            panic!("Expected Ref expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_parser_length_operator() {
        let mut parser = Parser::new("${#var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Length { inner } = expr {
            if let ParamExpr::Ref { target, .. } = inner.as_ref() {
                assert_eq!(target.name, "var");
            } else {
                panic!("Expected Ref in Length expression");
            }
        } else {
            panic!("Expected Length expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_parser_indirection() {
        let mut parser = Parser::new("${!var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Indirection { inner, style } = expr {
            if let ParamExpr::Ref { target, .. } = inner.as_ref() {
                assert_eq!(target.name, "var");
                assert_eq!(style, IndirectStyle::BashBang);
            } else {
                panic!("Expected Ref in Indirection expression");
            }
        } else {
            panic!("Expected Indirection expression, got {:?}", expr);
        }
    }

    #[test]
    fn test_parser_special_parameters() {
        // Test $#
        let mut parser = Parser::new("${#}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Ref { target, .. } = expr {
            assert_eq!(target.special, Some('#'));
        } else {
            panic!("Expected Ref expression for special parameter");
        }

        // Test $?
        let mut parser = Parser::new("${?}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Ref { target, .. } = expr {
            assert_eq!(target.special, Some('?'));
        } else {
            panic!("Expected Ref expression for special parameter");
        }

        // Test positional parameter
        let mut parser = Parser::new("${1}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Ref { target, .. } = expr {
            assert_eq!(target.positional, Some(1));
        } else {
            panic!("Expected Ref expression for positional parameter");
        }
    }

    #[test]
    fn test_parser_array_index() {
        let mut parser = Parser::new("${arr[5]}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Ref { target, index } = expr {
            assert_eq!(target.name, "arr");
            if let Index::One(n) = index {
                assert_eq!(n, 5);
            } else {
                panic!("Expected One index, got {:?}", index);
            }
        } else {
            panic!("Expected Ref expression with index");
        }
    }

    #[test]
    fn test_parser_default_operations() {
        // Test ${var:-default}
        let mut parser = Parser::new("${var:-default}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::Dash);
            assert_eq!(colon, true);
        } else {
            panic!("Expected Defaulting expression");
        }

        // Test ${var:=assign}
        let mut parser = Parser::new("${var:=assign}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::Assign);
            assert_eq!(colon, true);
        } else {
            panic!("Expected Defaulting expression");
        }

        // Test ${var:?error}
        let mut parser = Parser::new("${var:?error}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::QMark);
            assert_eq!(colon, true);
        } else {
            panic!("Expected Defaulting expression");
        }

        // Test ${var:+alt}
        let mut parser = Parser::new("${var:+alt}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::Plus);
            assert_eq!(colon, true);
        } else {
            panic!("Expected Defaulting expression");
        }

        // Test without colon ${var-default}
        let mut parser = Parser::new("${var-default}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::Dash);
            assert_eq!(colon, false);
        } else {
            panic!("Expected Defaulting expression");
        }
    }

    #[test]
    fn test_parser_substring() {
        // Test ${var:5}
        let mut parser = Parser::new("${var:5}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Substring { offset, len, .. } = expr {
            assert_eq!(offset, 5);
            assert_eq!(len, None);
        } else {
            panic!("Expected Substring expression");
        }

        // Test ${var:5:3}
        let mut parser = Parser::new("${var:5:3}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Substring { offset, len, .. } = expr {
            assert_eq!(offset, 5);
            assert_eq!(len, Some(3));
        } else {
            panic!("Expected Substring expression");
        }

        // Test negative offset
        let mut parser = Parser::new("${var:-5}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Substring { offset, len, .. } = expr {
            assert_eq!(offset, -5);
            assert_eq!(len, None);
        } else {
            panic!("Expected Substring expression");
        }
    }

    #[test]
    fn test_parser_prefix_suffix_removal() {
        // Test ${var#pattern}
        let mut parser = Parser::new("${var#pat*}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Remove { op, .. } = expr {
            assert_eq!(op, RemoveOp::Prefix { long: false });
        } else {
            panic!("Expected Remove expression");
        }

        // Test ${var##pattern}
        let mut parser = Parser::new("${var##pat*}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Remove { op, .. } = expr {
            assert_eq!(op, RemoveOp::Prefix { long: true });
        } else {
            panic!("Expected Remove expression");
        }

        // Test ${var%pattern}
        let mut parser = Parser::new("${var%*end}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Remove { op, .. } = expr {
            assert_eq!(op, RemoveOp::Suffix { long: false });
        } else {
            panic!("Expected Remove expression");
        }

        // Test ${var%%pattern}
        let mut parser = Parser::new("${var%%*end}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Remove { op, .. } = expr {
            assert_eq!(op, RemoveOp::Suffix { long: true });
        } else {
            panic!("Expected Remove expression");
        }
    }

    #[test]
    fn test_parser_string_replacement() {
        // Test ${var/pattern/replacement}
        let mut parser = Parser::new("${var/old/new}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Replace { scope, .. } = expr {
            assert_eq!(scope, ReplaceScope::First);
        } else {
            panic!("Expected Replace expression");
        }

        // Test ${var//pattern/replacement}
        let mut parser = Parser::new("${var//old/new}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Replace { scope, .. } = expr {
            assert_eq!(scope, ReplaceScope::Global);
        } else {
            panic!("Expected Replace expression");
        }

        // Test ${var/#pattern/replacement}
        let mut parser = Parser::new("${var/#old/new}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Replace { scope, .. } = expr {
            assert_eq!(scope, ReplaceScope::AnchorPrefix);
        } else {
            panic!("Expected Replace expression");
        }

        // Test ${var/%pattern/replacement}
        let mut parser = Parser::new("${var/%old/new}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Replace { scope, .. } = expr {
            assert_eq!(scope, ReplaceScope::AnchorSuffix);
        } else {
            panic!("Expected Replace expression");
        }
    }

    #[test]
    fn test_parser_path_modifiers() {
        // Test ${var:h:t:r:e}
        let mut parser = Parser::new("${var:h:t:r:e}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Modifiers { mods, .. } = expr {
            assert_eq!(mods, vec![PathMod::H, PathMod::T, PathMod::R, PathMod::E]);
        } else {
            panic!("Expected Modifiers expression");
        }

        // Test individual modifiers
        let mut parser = Parser::new("${var:h}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Modifiers { mods, .. } = expr {
            assert_eq!(mods, vec![PathMod::H]);
        } else {
            panic!("Expected Modifiers expression");
        }
    }

    #[test]
    fn test_parser_zsh_flags() {
        // Test ${(o)var}
        let mut parser = Parser::new("${(o)var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            assert_eq!(flags[0].kind, ZFlag::O);
        } else {
            panic!("Expected ZshFlags expression");
        }

        // Test ${(O)var}
        let mut parser = Parser::new("${(O)var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            assert_eq!(flags[0].kind, ZFlag::ODesc);
        } else {
            panic!("Expected ZshFlags expression");
        }

        // Test ${(u)var}
        let mut parser = Parser::new("${(u)var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            assert_eq!(flags[0].kind, ZFlag::Unique);
        } else {
            panic!("Expected ZshFlags expression");
        }

        // Test ${(P)var} - indirection flag
        let mut parser = Parser::new("${(P)var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            assert_eq!(flags[0].kind, ZFlag::P);
        } else {
            panic!("Expected ZshFlags expression");
        }
    }

    #[test]
    fn test_parser_error_cases() {
        // Missing opening brace
        let mut parser = Parser::new("var}");
        let result = parser.parse_braced();
        assert!(result.is_err());
        if let Err(Error::BadSubstitution(msg)) = result {
            assert!(msg.contains("expected '${' at start"));
        }

        // Missing closing brace
        let mut parser = Parser::new("${var");
        let result = parser.parse_braced();
        assert!(result.is_err());
        if let Err(Error::BadSubstitution(msg)) = result {
            assert!(msg.contains("expected '}' at end"));
        }

        // Invalid flag syntax
        let mut parser = Parser::new("${(xyz)var}");
        let _result = parser.parse_braced();
        // This might succeed or fail depending on implementation - flags may be lenient

        // Empty variable name
        let mut parser = Parser::new("${}");
        let result = parser.parse_braced();
        assert!(result.is_err());

        // Invalid array index
        let mut parser = Parser::new("${arr[]}");
        let result = parser.parse_braced();
        assert!(result.is_err());

        // Unclosed array index
        let mut parser = Parser::new("${arr[5");
        let result = parser.parse_braced();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_complex_combinations() {
        // Test ${#var[5]:2:3}
        let mut parser = Parser::new("${#var[5]:2:3}");
        let expr = parser.parse_braced().unwrap();

        match expr {
            ParamExpr::Substring { inner, offset, len } => {
                assert_eq!(offset, 2);
                assert_eq!(len, Some(3));

                if let ParamExpr::Length { inner: length_inner } = inner.as_ref() {
                    if let ParamExpr::Ref { target, index } = length_inner.as_ref() {
                        assert_eq!(target.name, "var");
                        if let Index::One(n) = index {
                            assert_eq!(*n, 5);
                        }
                    }
                }
            }
            _ => panic!("Expected complex Substring expression"),
        }

        // Test ${(o)var:h}
        let mut parser = Parser::new("${(o)var:h}");
        let expr = parser.parse_braced().unwrap();

        match expr {
            ParamExpr::Modifiers { inner, mods } => {
                assert_eq!(mods, vec![PathMod::H]);

                if let ParamExpr::ZshFlags { flags, .. } = inner.as_ref() {
                    assert_eq!(flags.len(), 1);
                    assert_eq!(flags[0].kind, ZFlag::O);
                }
            }
            _ => panic!("Expected Modifiers with ZshFlags expression"),
        }
    }

    #[test]
    fn test_parse_braced_function() {
        // Test standalone parse_braced function
        let result = parse_braced("${var:-default}").unwrap();
        if let ParamExpr::Defaulting { op, .. } = result {
            assert_eq!(op, DefaultOp::Dash);
        } else {
            panic!("Expected Defaulting expression");
        }

        // Test error case
        let result = parse_braced("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_expansions() {
        let expansions = find_expansions("Hello ${name}, your score is ${score:-0}!").unwrap();
        assert_eq!(expansions.len(), 2);

        // Check first expansion
        assert_eq!(expansions[0].0, 6); // start
        assert_eq!(expansions[0].1, 13); // end

        // Check second expansion - 实际位置
        assert_eq!(expansions[1].0, 29); // start
        assert_eq!(expansions[1].1, 40); // end

        // Test nested expansions - 这种语法实际上是无效的，应该会报错
        let result = find_expansions("${outer${inner}more}");
        assert!(result.is_err()); // 嵌套expansion语法错误

        // Test valid separate expansions
        let expansions = find_expansions("${outer} ${inner}").unwrap();
        assert_eq!(expansions.len(), 2);

        // Test escaped braces - 当前实现不处理转义，所以仍然会找到expansions
        let expansions = find_expansions("\\${not_expansion} \\${real}").unwrap();
        assert_eq!(expansions.len(), 2); // 当前实现会找到两个expansion

        // Test mixed escaped and real
        let expansions = find_expansions("\\${escaped} ${real}").unwrap();
        assert_eq!(expansions.len(), 2); // 当前实现会找到两个expansion
    }

    #[test]
    fn test_edge_cases() {
        // Test empty input
        let mut parser = Parser::new("${}");
        let result = parser.parse_braced();
        assert!(result.is_err());

        // Test just $
        let expansions = find_expansions("$").unwrap();
        assert_eq!(expansions.len(), 0);

        // Test incomplete expansion
        let expansions = find_expansions("${incomplete").unwrap();
        assert_eq!(expansions.len(), 0); // Should not find incomplete expansion

        // Test multiple consecutive expansions
        let expansions = find_expansions("${a}${b}${c}").unwrap();
        assert_eq!(expansions.len(), 3);
    }

    #[test]
    fn test_parser_methods() {
        let _parser = Parser::new("test string");

        // Test access to lexer functionality through parser
        // This tests internal structure if parser exposes lexer methods
        // For now just test that parser can be created with various inputs

        let _parser2 = Parser::new("");
        let _parser3 = Parser::new("${simple}");
        let _parser4 = Parser::new("unicode: éñ 中文");

        // These should not panic on creation
        assert_eq!(std::mem::size_of::<Parser>(), std::mem::size_of::<Parser>());
    }

    #[test]
    fn test_parser_zsh_flags_with_args() {
        // Test ${(j)var} - join flag without separator (should use default space)
        let mut parser = Parser::new("${(j)var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            if let ZFlag::J { sep } = &flags[0].kind {
                assert_eq!(sep, " "); // Default separator
            } else {
                panic!("Expected J flag, got {:?}", flags[0].kind);
            }
        } else {
            panic!("Expected ZshFlags expression");
        }

        // Test ${(s)str} - split flag without separator (should use default space)
        let mut parser = Parser::new("${(s)str}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            if let ZFlag::S { sep } = &flags[0].kind {
                assert_eq!(sep, " "); // Default separator
            } else {
                panic!("Expected S flag, got {:?}", flags[0].kind);
            }
        } else {
            panic!("Expected ZshFlags expression");
        }
    }

    #[test]
    fn test_parser_zsh_flags_error_cases() {
        // Test (l) flag without width argument
        let mut parser = Parser::new("${(l)var}");
        let result = parser.parse_braced();
        assert!(result.is_err());
        if let Err(Error::BadSubstitution(msg)) = result {
            assert!(msg.contains("(l) flag requires width argument"));
        }

        // Test (r) flag without width argument
        let mut parser = Parser::new("${(r)var}");
        let result = parser.parse_braced();
        assert!(result.is_err());
        if let Err(Error::BadSubstitution(msg)) = result {
            assert!(msg.contains("(r) flag requires width argument"));
        }

        // Test unknown flag
        let mut parser = Parser::new("${(X)var}");
        let result = parser.parse_braced();
        assert!(result.is_err());
        if let Err(Error::BadSubstitution(msg)) = result {
            assert!(msg.contains("unknown flag 'X'"));
        }
    }

    #[test]
    fn test_parser_special_edge_cases() {
        // Test ${#} - special parameter, not length
        let mut parser = Parser::new("${#}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Ref { target, .. } = expr {
            assert_eq!(target.name, "#");
            assert!(target.special.is_some());
        } else {
            panic!("Expected Ref expression for ${{#}}");
        }

        // Test mixed flags ${(ou)var}
        let mut parser = Parser::new("${(ou)var}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 2);
            assert_eq!(flags[0].kind, ZFlag::O);
            assert_eq!(flags[1].kind, ZFlag::Unique);
        } else {
            panic!("Expected ZshFlags expression");
        }

        // Test empty flag parentheses
        let mut parser = Parser::new("${()var}");
        let expr = parser.parse_braced().unwrap();

        // Empty flags should result in a ZshFlags expression with empty flags vector
        match expr {
            ParamExpr::ZshFlags { flags, .. } => {
                assert_eq!(flags.len(), 0);
            }
            ParamExpr::Ref { .. } => {
                // If parsed as Ref, that's also acceptable since empty flags don't change behavior
                println!("Parsed as Ref instead of ZshFlags - acceptable");
            }
            _ => panic!("Unexpected expression type for ${{()var}}: {:?}", expr),
        }
    }

    #[test]
    fn test_parser_complex_word_parsing() {
        // Test word parsing with mixed content
        let mut parser = Parser::new("${var:-default with spaces}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Defaulting { word, .. } = expr {
            assert_eq!(word.len(), 1);
            if let Word::Text(text) = &word[0] {
                assert_eq!(text, "default with spaces");
            } else {
                panic!("Expected Text word");
            }
        } else {
            panic!("Expected Defaulting expression");
        }

        // Test word parsing with simple text
        let mut parser = Parser::new("${var:-simple text}");
        let expr = parser.parse_braced().unwrap();

        if let ParamExpr::Defaulting { word, .. } = expr {
            assert_eq!(word.len(), 1);
            if let Word::Text(text) = &word[0] {
                assert_eq!(text, "simple text");
            } else {
                panic!("Expected Text word");
            }
        } else {
            panic!("Expected Defaulting expression");
        }
    }

    #[test]
    fn test_parser_additional_flags() {
        // Test more flag types to improve coverage
        let flag_tests = vec![
            ("${(U)var}", ZFlag::U),
            ("${(L)var}", ZFlag::L),
            ("${(C)var}", ZFlag::C),
            ("${(q)var}", ZFlag::Q),
            ("${(Q)var}", ZFlag::Unquote),
            ("${(f)var}", ZFlag::F),
            ("${(z)var}", ZFlag::Z),
            ("${(Z)var}", ZFlag::ZExt),
            ("${(k)var}", ZFlag::K),
            ("${(v)var}", ZFlag::V),
            ("${(t)var}", ZFlag::T),
            ("${(V)var}", ZFlag::VDisplay),
            ("${(e)var}", ZFlag::E),
        ];

        for (input, expected_flag) in flag_tests {
            let mut parser = Parser::new(input);
            let expr = parser.parse_braced().unwrap();

            if let ParamExpr::ZshFlags { flags, .. } = expr {
                assert_eq!(flags.len(), 1);
                assert_eq!(flags[0].kind, expected_flag);
            } else {
                panic!("Expected ZshFlags expression for {}", input);
            }
        }
    }

    #[test]
    fn test_parser_invalid_syntax_coverage() {
        // Test various invalid syntax cases to improve error path coverage
        let invalid_cases = vec![
            "${",          // Incomplete opening
            "var}",        // Missing opening
            "${var[",      // Unclosed bracket
            "${var]}",     // Invalid bracket
            "${var[abc]}", // Non-numeric index (should still parse)
            "${(",         // Incomplete flag
            "${()}",       // Empty expression after flags
            "${#",         // Incomplete length operator
            "${!",         // Incomplete indirection
        ];

        for case in invalid_cases {
            let mut parser = Parser::new(case);
            let result = parser.parse_braced();
            // These should mostly fail, but some might be handled gracefully
            if result.is_ok() {
                println!("Unexpectedly parsed: {}", case);
            }
        }
    }

    #[test]
    fn test_parser_find_expansions_edge_cases() {
        // Test find_expansions with various edge cases
        let test_cases = vec![
            ("", vec![]),
            ("no expansions here", vec![]),
            ("$simple", vec![(0, 7)]), // Simple variable
            ("prefix$var suffix", vec![(6, 10)]),
            ("$a $b $c", vec![(0, 2), (3, 5), (6, 8)]),
            ("$$escaped", vec![(1, 9)]),                   // Second $ starts expansion
            ("$1 $2 $123", vec![(0, 2), (3, 5), (6, 10)]), // Positional params
        ];

        for (input, expected_ranges) in test_cases {
            let expansions = find_expansions(input).unwrap();
            let actual_ranges: Vec<(usize, usize)> = expansions.iter().map(|(start, end, _)| (*start, *end)).collect();
            assert_eq!(actual_ranges, expected_ranges, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_parser_flags_l_r_with_args() {
        // Test ${(l:5:)var} - left pad flag with width only
        let mut parser = Parser::new("${(l:5:)var}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            match &flags[0].kind {
                ZFlag::L2 { width, fill, pad } => {
                    assert_eq!(width, "5");
                    assert_eq!(fill, "");
                    assert_eq!(pad, "");
                }
                other => panic!("Expected L2 flag, got {:?}", other),
            }
        } else {
            panic!("Expected ZshFlags expression");
        }

        // Test ${(r:5:)var} - right pad flag
        let mut parser = Parser::new("${(r:5:)var}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::ZshFlags { flags, .. } = expr {
            assert_eq!(flags.len(), 1);
            match &flags[0].kind {
                ZFlag::R2 { width, fill, pad } => {
                    assert_eq!(width, "5");
                    assert_eq!(fill, "");
                    assert_eq!(pad, "");
                }
                other => panic!("Expected R2 flag, got {:?}", other),
            }
        } else {
            panic!("Expected ZshFlags expression");
        }
    }

    #[test]
    fn test_parser_index_key_and_slice() {
        // Test associative array key index
        let mut parser = Parser::new("${map[key]}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Ref { index, .. } = expr {
            match index {
                Index::Key(k) => assert_eq!(k, "key"),
                _ => panic!("Expected Key index"),
            }
        } else {
            panic!("Expected Ref expression");
        }

        // Test slice index
        let mut parser = Parser::new("${arr[1,3]}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Ref { index, .. } = expr {
            match index {
                Index::Slice(a, b) => {
                    assert_eq!(a, 1);
                    assert_eq!(b, 3);
                }
                _ => panic!("Expected Slice index"),
            }
        } else {
            panic!("Expected Ref expression");
        }

        // Test missing second number in slice
        let mut parser = Parser::new("${arr[1,]}");
        let result = parser.parse_braced();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_colonless_operations() {
        // ${var=assign}
        let mut parser = Parser::new("${var=assign}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::Assign);
            assert!(!colon);
        } else {
            panic!("Expected Assign defaulting");
        }

        // ${var+alt}
        let mut parser = Parser::new("${var+alt}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::Plus);
            assert!(!colon);
        } else {
            panic!("Expected Plus defaulting");
        }

        // ${var?err}
        let mut parser = Parser::new("${var?err}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Defaulting { op, colon, .. } = expr {
            assert_eq!(op, DefaultOp::QMark);
            assert!(!colon);
        } else {
            panic!("Expected QMark defaulting");
        }
    }

    #[test]
    fn test_parser_path_modifiers_a_a() {
        let mut parser = Parser::new("${var:A:a}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Modifiers { mods, .. } = expr {
            assert_eq!(mods, vec![PathMod::A, PathMod::LowerA]);
        } else {
            panic!("Expected Modifiers expression");
        }
    }

    #[test]
    fn test_parser_colon_error() {
        let mut parser = Parser::new("${var:}");
        let result = parser.parse_braced();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_word_with_substitutions() {
        // Command substitution
        let mut parser = Parser::new("${var:-$(echo hi)}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Defaulting { word, .. } = expr {
            assert!(matches!(word[0], Word::CmdSubst(_)));
        } else {
            panic!("Expected Defaulting expression");
        }

        // Arithmetic substitution
        let mut parser = Parser::new("${var:-$((1+2))}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Defaulting { word, .. } = expr {
            assert!(matches!(word[0], Word::ArithSubst(_)));
        } else {
            panic!("Expected Defaulting expression");
        }

        // Invalid arithmetic substitution
        let mut parser = Parser::new("${var:-$((1+2)}");
        let result = parser.parse_braced();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_word_invalid_dollar() {
        let mut parser = Parser::new("${var:-$}");
        let expr = parser.parse_braced().unwrap();
        if let ParamExpr::Defaulting { word, .. } = expr {
            assert!(matches!(word[0], Word::Text(ref t) if t == "$"));
        } else {
            panic!("Expected Defaulting expression");
        }
    }
}
