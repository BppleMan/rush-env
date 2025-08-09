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
        while self.peek().map_or(false, |c| c.is_whitespace()) {
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
            } else {
                return String::new();
            }
        }

        // Following chars can be alphanumeric or underscore
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.next_char();
            } else {
                break;
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
