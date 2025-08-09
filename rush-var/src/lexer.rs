use crate::ast::*;

/// Lexer for tokenizing parameter expansion content
#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Get current position
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Peek at current character without consuming
    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    /// Peek ahead n characters
    pub fn peek_ahead(&self, n: usize) -> Option<char> {
        self.input.chars().nth(self.pos + n)
    }

    /// Consume and return current character
    pub fn next_char(&mut self) -> Option<char> {
        if let Some(ch) = self.input.chars().nth(self.pos) {
            self.pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    /// Skip whitespace
    pub fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.next_char();
        }
    }

    /// Check if at end of input
    pub fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Get remaining input
    pub fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    /// Read identifier (variable name)
    pub fn read_identifier(&mut self) -> String {
        let start = self.pos;

        // First char must be letter or underscore
        if let Some(ch) = self.peek() {
            if ch.is_alphabetic() || ch == '_' {
                self.next_char();

                // Following chars can be alphanumeric or underscore
                while let Some(ch) = self.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        self.next_char();
                    } else {
                        break;
                    }
                }
            }
        }

        self.input[start..self.pos].to_string()
    }

    /// Read number
    pub fn read_number(&mut self) -> Option<i64> {
        let start = self.pos;
        let mut has_sign = false;

        // Optional sign
        if let Some(ch) = self.peek() {
            if ch == '-' || ch == '+' {
                self.next_char();
                has_sign = true;
            }
        }

        // Read digits
        let mut found_digit = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.next_char();
                found_digit = true;
            } else {
                break;
            }
        }

        if found_digit || (has_sign && start + 1 < self.pos) {
            self.input[start..self.pos].parse().ok()
        } else {
            // Reset position if no valid number
            self.pos = start;
            None
        }
    }

    /// Read until unescaped delimiter
    pub fn read_until_unescaped(&mut self, delim: char) -> Result<String, Error> {
        let mut result = String::new();
        let mut escaped = false;

        while let Some(ch) = self.peek() {
            if escaped {
                result.push(ch);
                self.next_char();
                escaped = false;
            } else if ch == '\\' {
                result.push(ch);
                self.next_char();
                escaped = true;
            } else if ch == delim {
                break;
            } else {
                result.push(ch);
                self.next_char();
            }
        }

        Ok(result)
    }

    /// Read until any of the specified delimiters
    pub fn read_until_any(&mut self, delims: &[char]) -> String {
        let mut result = String::new();
        let mut escaped = false;

        while let Some(ch) = self.peek() {
            if escaped {
                result.push(ch);
                self.next_char();
                escaped = false;
            } else if ch == '\\' {
                result.push(ch);
                self.next_char();
                escaped = true;
            } else if delims.contains(&ch) {
                break;
            } else {
                result.push(ch);
                self.next_char();
            }
        }

        result
    }

    /// Read flag arguments separated by colons
    pub fn read_flag_args(&mut self, start_delim: char, end_delim: char) -> Vec<String> {
        let mut args = Vec::new();

        if self.peek() == Some(start_delim) {
            self.next_char(); // consume start delimiter

            loop {
                let arg = self.read_until_any(&[start_delim, end_delim]);
                args.push(arg);

                match self.peek() {
                    Some(ch) if ch == start_delim && ch != end_delim => {
                        self.next_char(); // consume separator
                        continue;
                    }
                    Some(ch) if ch == end_delim => {
                        self.next_char(); // consume end delimiter
                        break;
                    }
                    Some(ch) if ch == start_delim && ch == end_delim => {
                        // When start and end delims are the same, we treat this as end delimiter
                        // after reading at least one argument
                        self.next_char(); // consume delimiter
                        break;
                    }
                    _ => break,
                }
            }
        }

        args
    }

    /// Expect and consume specific character
    pub fn expect(&mut self, expected: char) -> Result<(), Error> {
        match self.next_char() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(Error::BadSubstitution(format!("expected '{}', found '{}'", expected, ch))),
            None => Err(Error::BadSubstitution(format!("expected '{}', found end of input", expected))),
        }
    }

    /// Check if next character matches without consuming
    pub fn matches(&self, ch: char) -> bool {
        self.peek() == Some(ch)
    }

    /// Check if next characters match a string
    pub fn matches_str(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    /// Consume string if it matches
    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.matches_str(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lexer_basic_operations() {
        let mut lexer = Lexer::new("hello");
        assert_eq!(lexer.pos(), 0);
        assert_eq!(lexer.peek(), Some('h'));
        assert_eq!(lexer.next_char(), Some('h'));
        assert_eq!(lexer.pos(), 1);
        assert_eq!(lexer.peek(), Some('e'));
        assert!(!lexer.is_at_end());
    }

    #[test]
    fn test_lexer_utf8() {
        let mut lexer = Lexer::new("héllo");
        assert_eq!(lexer.next_char(), Some('h'));
        assert_eq!(lexer.next_char(), Some('é')); // Multi-byte character
        assert_eq!(lexer.pos(), 3); // h(1) + é(2)
        assert_eq!(lexer.peek(), Some('l'));
    }

    #[test]
    fn test_lexer_peek_ahead() {
        let lexer = Lexer::new("hello");
        assert_eq!(lexer.peek_ahead(0), Some('h'));
        assert_eq!(lexer.peek_ahead(1), Some('e'));
        assert_eq!(lexer.peek_ahead(4), Some('o'));
        assert_eq!(lexer.peek_ahead(5), None);
    }

    #[test]
    fn test_skip_whitespace() {
        let mut lexer = Lexer::new("   \t\n  hello");
        lexer.skip_whitespace();
        assert_eq!(lexer.peek(), Some('h'));

        let mut empty_lexer = Lexer::new("   \t\n  ");
        empty_lexer.skip_whitespace();
        assert!(empty_lexer.is_at_end());
    }

    #[test]
    fn test_read_identifier() {
        let mut lexer = Lexer::new("var_name123 ");
        let ident = lexer.read_identifier();
        assert_eq!(ident, "var_name123");
        assert_eq!(lexer.peek(), Some(' '));

        // Test edge cases
        let mut lexer = Lexer::new("_underscore");
        assert_eq!(lexer.read_identifier(), "_underscore");

        let mut lexer = Lexer::new("123invalid");
        assert_eq!(lexer.read_identifier(), ""); // Should not start with digit

        let mut lexer = Lexer::new("");
        assert_eq!(lexer.read_identifier(), "");
    }

    #[test]
    fn test_read_number() {
        // Positive numbers
        let mut lexer = Lexer::new("123");
        assert_eq!(lexer.read_number(), Some(123));
        assert!(lexer.is_at_end());

        // Negative numbers
        let mut lexer = Lexer::new("-456");
        assert_eq!(lexer.read_number(), Some(-456));

        // Numbers with plus sign
        let mut lexer = Lexer::new("+789");
        assert_eq!(lexer.read_number(), Some(789));

        // Invalid numbers
        let mut lexer = Lexer::new("abc");
        assert_eq!(lexer.read_number(), None);
        assert_eq!(lexer.pos(), 0); // Position should be reset

        // Just a sign
        let mut lexer = Lexer::new("-");
        assert_eq!(lexer.read_number(), None);

        // Sign followed by non-digit
        let mut lexer = Lexer::new("-abc");
        assert_eq!(lexer.read_number(), None);
        assert_eq!(lexer.pos(), 0); // Position should be reset

        // Number followed by non-digit
        let mut lexer = Lexer::new("123abc");
        assert_eq!(lexer.read_number(), Some(123));
        assert_eq!(lexer.peek(), Some('a'));
    }

    #[test]
    fn test_read_until_unescaped() {
        let mut lexer = Lexer::new("hello:world");
        let result = lexer.read_until_unescaped(':').unwrap();
        assert_eq!(result, "hello");
        assert_eq!(lexer.peek(), Some(':'));

        // Test with escapes
        let mut lexer = Lexer::new("hello\\:world:end");
        let result = lexer.read_until_unescaped(':').unwrap();
        assert_eq!(result, "hello\\:world");
        assert_eq!(lexer.peek(), Some(':'));

        // Test with no delimiter
        let mut lexer = Lexer::new("hello world");
        let result = lexer.read_until_unescaped(':').unwrap();
        assert_eq!(result, "hello world");
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_read_until_any() {
        let mut lexer = Lexer::new("hello:world;end");
        let result = lexer.read_until_any(&[':', ';']);
        assert_eq!(result, "hello");
        assert_eq!(lexer.peek(), Some(':'));

        // Test with escapes
        let mut lexer = Lexer::new("hello\\:world;end");
        let result = lexer.read_until_any(&[':', ';']);
        assert_eq!(result, "hello\\:world");
        assert_eq!(lexer.peek(), Some(';'));

        // Test with no delimiters
        let mut lexer = Lexer::new("hello world");
        let result = lexer.read_until_any(&[':', ';']);
        assert_eq!(result, "hello world");
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_read_flag_args() {
        // Basic case - 当start和end分隔符相同时，只能读取一个参数
        let mut lexer = Lexer::new(":arg1:arg2:arg3:");
        let args = lexer.read_flag_args(':', ':');
        assert_eq!(args, vec!["arg1"]); // 相同分隔符时只读取第一个参数

        // No args
        let mut lexer = Lexer::new("::");
        let args = lexer.read_flag_args(':', ':');
        assert_eq!(args, vec![""]);

        // Different delimiters - 可以读取完整内容
        let mut lexer = Lexer::new("{arg1:arg2:arg3}");
        let args = lexer.read_flag_args('{', '}');
        assert_eq!(args, vec!["arg1:arg2:arg3"]);

        // No start delimiter
        let mut lexer = Lexer::new("arg1:arg2");
        let args = lexer.read_flag_args(':', ':');
        assert_eq!(args, Vec::<String>::new());

        // Missing end delimiter - 当分隔符相同时只能读取一个参数
        let mut lexer = Lexer::new(":arg1:arg2");
        let args = lexer.read_flag_args(':', ':');
        assert_eq!(args, vec!["arg1"]); // 第二个:被当作结束符
    }

    #[test]
    fn test_expect() {
        let mut lexer = Lexer::new("hello");
        assert!(lexer.expect('h').is_ok());
        assert_eq!(lexer.peek(), Some('e'));

        // Wrong character
        let mut lexer = Lexer::new("hello");
        let result = lexer.expect('x');
        assert!(result.is_err());
        if let Err(Error::BadSubstitution(msg)) = result {
            assert!(msg.contains("expected 'x', found 'h'"));
        }

        // End of input
        let mut lexer = Lexer::new("");
        let result = lexer.expect('x');
        assert!(result.is_err());
        if let Err(Error::BadSubstitution(msg)) = result {
            assert!(msg.contains("expected 'x', found end of input"));
        }
    }

    #[test]
    fn test_matches() {
        let lexer = Lexer::new("hello");
        assert!(lexer.matches('h'));
        assert!(!lexer.matches('x'));

        let empty_lexer = Lexer::new("");
        assert!(!empty_lexer.matches('h'));
    }

    #[test]
    fn test_matches_str() {
        let lexer = Lexer::new("hello world");
        assert!(lexer.matches_str("hello"));
        assert!(lexer.matches_str("hello "));
        assert!(!lexer.matches_str("hi"));
        assert!(!lexer.matches_str("hello world!"));

        let empty_lexer = Lexer::new("");
        assert!(!empty_lexer.matches_str("hello"));
        assert!(empty_lexer.matches_str("")); // Empty string should match
    }

    #[test]
    fn test_consume_str() {
        let mut lexer = Lexer::new("hello world");
        assert!(lexer.consume_str("hello"));
        assert_eq!(lexer.peek(), Some(' '));
        assert_eq!(lexer.pos(), 5);

        assert!(!lexer.consume_str("hi"));
        assert_eq!(lexer.pos(), 5); // Position unchanged

        assert!(lexer.consume_str(" world"));
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_remaining() {
        let mut lexer = Lexer::new("hello world");
        assert_eq!(lexer.remaining(), "hello world");

        lexer.next_char(); // consume 'h'
        assert_eq!(lexer.remaining(), "ello world");

        while !lexer.is_at_end() {
            lexer.next_char();
        }
        assert_eq!(lexer.remaining(), "");
    }

    #[test]
    fn test_complex_scenarios() {
        // Test mixed operations
        let mut lexer = Lexer::new("  var123:456  ");
        lexer.skip_whitespace();

        let ident = lexer.read_identifier();
        assert_eq!(ident, "var123");

        assert!(lexer.expect(':').is_ok());

        let num = lexer.read_number().unwrap();
        assert_eq!(num, 456);

        lexer.skip_whitespace();
        assert!(lexer.is_at_end());
    }
}
