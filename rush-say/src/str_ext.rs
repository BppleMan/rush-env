use crate::widget::visual_width_char;
use std::ops::Range;

pub trait StrExt {
    fn detect_newline_kind(s: &str) -> Option<NewlineKind>;

    fn visual_width(&self) -> usize;

    fn lines_with_range(&self) -> Vec<(&str, Range<usize>)>;
}

pub enum NewlineKind {
    LF,
    CRLF,
    MIXED,
}

impl StrExt for str {
    fn detect_newline_kind(s: &str) -> Option<NewlineKind> {
        let has_lf = s.contains('\n');
        let has_crlf = s.contains("\r\n");

        match (has_lf, has_crlf) {
            (false, false) => None,
            (true, true) => Some(NewlineKind::MIXED),
            (true, false) => Some(NewlineKind::LF),
            (false, true) => Some(NewlineKind::CRLF),
        }
    }

    fn visual_width(&self) -> usize {
        self.chars().map(visual_width_char).sum()
    }

    fn lines_with_range(&self) -> Vec<(&str, Range<usize>)> {
        let bytes = self.as_bytes();
        let mut lines = Vec::new();

        let mut start = 0;
        let mut i = 0;

        while i < bytes.len() {
            match bytes[i] {
                b'\n' => {
                    // handle '\n'
                    let range = start..i;
                    lines.push((&self[range.clone()], range));
                    i += 1;
                    start = i;
                }
                b'\r' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                        // handle '\r\n'
                        let range = start..i;
                        lines.push((&self[range.clone()], range));
                        i += 2;
                    } else {
                        // handle lone '\r'
                        let range = start..i;
                        lines.push((&self[range.clone()], range));
                        i += 1;
                    }
                    start = i;
                }
                _ => {
                    i += 1;
                }
            }
        }

        // 最后一行（可能没有换行符）
        if start < self.len() {
            let range = start..self.len();
            lines.push((&self[range.clone()], range));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::visual_width_char;

    #[test]
    fn test_visual_width_ascii() {
        assert_eq!("hello".visual_width(), 5);
        assert_eq!("".visual_width(), 0);
    }

    #[test]
    fn test_visual_width_unicode() {
        // 假设 visual_width_char('你') == 2
        let width_ni = visual_width_char('你');
        let width_hao = visual_width_char('好');
        assert_eq!("你".visual_width(), width_ni);
        assert_eq!("你好".visual_width(), width_ni + width_hao);
    }

    #[test]
    fn test_lines_with_range_basic() {
        let s = "hello\nworld";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "hello");
        assert_eq!(lines[0].1, 0..5);
        assert_eq!(lines[1].0, "world");
        assert_eq!(lines[1].1, 6..11);
    }

    #[test]
    fn test_lines_with_range_crlf() {
        let s = "foo\r\nbar\r\nbaz";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].0, "foo");
        assert_eq!(lines[0].1, 0..3);
        assert_eq!(lines[1].0, "bar");
        assert_eq!(lines[1].1, 5..8);
        assert_eq!(lines[2].0, "baz");
        assert_eq!(lines[2].1, 10..13);
    }

    #[test]
    fn test_lines_with_range_lone_cr() {
        let s = "a\rb\rc";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].0, "a");
        assert_eq!(lines[0].1, 0..1);
        assert_eq!(lines[1].0, "b");
        assert_eq!(lines[1].1, 2..3);
        assert_eq!(lines[2].0, "c");
        assert_eq!(lines[2].1, 4..5);
    }

    #[test]
    fn test_lines_with_range_trailing_newline() {
        let s = "foo\n";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "foo");
        assert_eq!(lines[0].1, 0..3);
    }

    #[test]
    fn test_lines_with_range_empty() {
        let s = "";
        let lines = s.lines_with_range();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_lines_with_range_spaces_only() {
        let s = "   ";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "   ");
        assert_eq!(lines[0].1, 0..3);
    }

    #[test]
    fn test_lines_with_range_leading_spaces() {
        let s = "  foo\n  bar";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "  foo");
        assert_eq!(lines[0].1, 0..5);
        assert_eq!(lines[1].0, "  bar");
        assert_eq!(lines[1].1, 6..11);
    }

    #[test]
    fn test_lines_with_range_trailing_spaces() {
        let s = "foo  \nbar  ";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "foo  ");
        assert_eq!(lines[0].1, 0..5);
        assert_eq!(lines[1].0, "bar  ");
        assert_eq!(lines[1].1, 6..11);
    }

    #[test]
    fn test_lines_with_range_crlf_with_spaces() {
        let s = "foo \r\n bar\r\n  baz ";
        let lines = s.lines_with_range();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].0, "foo ");
        assert_eq!(lines[0].1, 0..4);
        assert_eq!(lines[1].0, " bar");
        assert_eq!(lines[1].1, 6..10);
        assert_eq!(lines[2].0, "  baz ");
        assert_eq!(lines[2].1, 12..18);
    }
}
