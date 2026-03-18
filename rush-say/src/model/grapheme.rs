use crate::error::layout_error::SourceSpan;
use rush_ext::Getter;
use std::ops::{Add, AddAssign, Index};
use std::slice::Iter;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Default, Debug, Clone, Getter)]
pub struct Graphemes {
    content: String,
    graphemes: Vec<Grapheme>,
}

impl Graphemes {
    pub fn new(content: String) -> Self {
        let segment_indices = content.grapheme_indices(true);
        let graphemes = segment_indices
            .map(|(start, segment)| {
                let end = start + segment.len();
                let ascii_width = segment.width();
                Grapheme { start, end, ascii_width }
            })
            .collect::<Vec<_>>();
        Self { content, graphemes }
    }

    pub fn len(&self) -> usize {
        self.graphemes.len()
    }

    pub fn iter(&self) -> GraphemesIter<'_> {
        GraphemesIter {
            content: &self.content,
            graphemes: self.graphemes.iter(),
        }
    }
}

impl Index<usize> for Graphemes {
    type Output = Grapheme;

    fn index(&self, index: usize) -> &Self::Output {
        &self.graphemes[index]
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Grapheme {
    start: usize,
    end: usize,
    ascii_width: usize,
}

impl Grapheme {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn ascii_width(&self) -> usize {
        self.ascii_width
    }

    pub fn render<'a>(&'a self, graphemes: &'a Graphemes) -> &'a str {
        &graphemes.content[self.start..self.end]
    }
}

impl Add for Grapheme {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut merged = self;
        merged.ascii_width += rhs.ascii_width;
        if self.start == self.end {
            merged.start = rhs.start;
        }
        merged.end = rhs.end;
        merged
    }
}

impl AddAssign for Grapheme {
    fn add_assign(&mut self, rhs: Self) {
        self.ascii_width += rhs.ascii_width;
        if self.start == self.end {
            self.start = rhs.start;
        }
        self.end = rhs.end;
    }
}

#[derive(Debug, Clone)]
pub struct GraphemeText<'a> {
    text: &'a str,
    grapheme: &'a Grapheme,
}

impl GraphemeText<'_> {
    pub fn as_str(&self) -> &str {
        self.text
    }

    pub fn grapheme(&self) -> Grapheme {
        *self.grapheme
    }

    pub fn start(&self) -> usize {
        self.grapheme.start()
    }

    pub fn end(&self) -> usize {
        self.grapheme.end()
    }

    pub fn ascii_width(&self) -> usize {
        self.grapheme.ascii_width
    }
}

impl AsRef<str> for GraphemeText<'_> {
    fn as_ref(&self) -> &str {
        self.text
    }
}

pub struct GraphemesIter<'a> {
    content: &'a str,
    graphemes: Iter<'a, Grapheme>,
}

impl<'a> Iterator for GraphemesIter<'a> {
    type Item = GraphemeText<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let segment = self.graphemes.next()?;
        let text = &self.content[segment.start..segment.end];
        Some(GraphemeText { text, grapheme: segment })
    }
}

impl<'a> IntoIterator for &'a Graphemes {
    type Item = GraphemeText<'a>;
    type IntoIter = GraphemesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GraphemesIter {
            content: &self.content,
            graphemes: self.graphemes.iter(),
        }
    }
}

impl<'a> From<&'a GraphemeText<'a>> for SourceSpan {
    fn from(value: &'a GraphemeText<'a>) -> Self {
        SourceSpan::new(value.start(), value.as_str().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::{eyre::eyre, Result};

    fn assert_segments(graphemes: &Graphemes, expected: &[&str]) {
        let actual = graphemes.iter().map(|segment_text| segment_text.text).collect::<Vec<_>>();
        assert_eq!(actual, expected);
        assert_eq!(graphemes.len(), expected.len());
    }

    #[test]
    fn test_index_access_returns_expected_segments() -> Result<()> {
        let _ = color_eyre::install();

        let text = "Hello, 世界! 👋".to_string();
        let graphemes = Graphemes::new(text);
        assert_eq!(graphemes.len(), 12);
        assert_eq!(graphemes[0].render(&graphemes), "H");
        assert_eq!(graphemes[1].render(&graphemes), "e");
        assert_eq!(graphemes[2].render(&graphemes), "l");
        assert_eq!(graphemes[3].render(&graphemes), "l");
        assert_eq!(graphemes[4].render(&graphemes), "o");
        assert_eq!(graphemes[5].render(&graphemes), ",");
        assert_eq!(graphemes[6].render(&graphemes), " ");
        assert_eq!(graphemes[7].render(&graphemes), "世");
        assert_eq!(graphemes[8].render(&graphemes), "界");
        assert_eq!(graphemes[9].render(&graphemes), "!");
        assert_eq!(graphemes[10].render(&graphemes), " ");
        assert_eq!(graphemes[11].render(&graphemes), "👋");

        Ok(())
    }

    #[test]
    fn test_iter_returns_expected_segments() -> Result<()> {
        let _ = color_eyre::install();

        let text = "Hello, 世界! 👋".to_string();
        let graphemes = Graphemes::new(text);
        let collected = graphemes.iter().map(|gt| gt.text).collect::<Vec<_>>();
        assert_eq!(collected, vec!["H", "e", "l", "l", "o", ",", " ", "世", "界", "!", " ", "👋"]);

        Ok(())
    }

    #[test]
    fn test_empty_content() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new(String::new());
        assert_eq!(graphemes.len(), 0);
        assert!(graphemes.iter().next().is_none());
        assert_eq!((&graphemes).into_iter().count(), 0);

        Ok(())
    }

    #[test]
    fn test_ascii_width_for_mixed_content() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("A世👋".to_string());
        assert_segments(&graphemes, &["A", "世", "👋"]);
        assert_eq!(graphemes[0].ascii_width(), 1);
        assert_eq!(graphemes[1].ascii_width(), 2);
        assert_eq!(graphemes[2].ascii_width(), 2);

        Ok(())
    }

    #[test]
    fn test_iter_and_index_are_consistent() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("ab世👋z".to_string());
        for (i, iter_text) in graphemes.iter().enumerate() {
            let by_index = graphemes[i].render(&graphemes);
            assert_eq!(iter_text.as_str(), by_index);
            assert_eq!(iter_text.start(), graphemes[i].start());
            assert_eq!(iter_text.end(), graphemes[i].end());
            assert_eq!(iter_text.ascii_width(), graphemes[i].ascii_width());
        }

        Ok(())
    }

    #[test]
    fn test_newline_segments() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("a\n\nb".to_string());
        assert_segments(&graphemes, &["a", "\n", "\n", "b"]);

        Ok(())
    }

    #[test]
    fn test_combining_mark_is_single_grapheme() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("e\u{301}".to_string());
        assert_segments(&graphemes, &["e\u{301}"]);
        assert_eq!(graphemes[0].ascii_width(), 1);

        Ok(())
    }

    #[test]
    fn test_zwj_emoji_is_single_grapheme() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("🧑‍🚀".to_string());
        assert_segments(&graphemes, &["🧑‍🚀"]);
        assert!(graphemes[0].ascii_width() >= 2);

        Ok(())
    }

    #[test]
    fn test_flag_emoji_is_single_grapheme() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("🇺🇳".to_string());
        assert_segments(&graphemes, &["🇺🇳"]);
        assert!(graphemes[0].ascii_width() >= 2);

        Ok(())
    }

    #[test]
    fn test_variation_selector_is_single_grapheme() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("✌️".to_string());
        assert_segments(&graphemes, &["✌️"]);
        assert!(graphemes[0].ascii_width() >= 1);

        Ok(())
    }

    #[test]
    fn test_add_assign_accumulates_range_and_width() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("A世👋".to_string());
        let mut merged = Grapheme::default();

        let first = graphemes.iter().next().ok_or_else(|| eyre!("missing first grapheme"))?;
        merged += first.grapheme();
        assert_eq!(merged.start(), first.start());
        assert_eq!(merged.end(), first.end());
        assert_eq!(merged.ascii_width(), first.ascii_width());

        let second = graphemes.iter().nth(1).ok_or_else(|| eyre!("missing second grapheme"))?;
        merged += second.grapheme();
        assert_eq!(merged.render(&graphemes), "A世");
        assert_eq!(merged.ascii_width(), first.ascii_width() + second.ascii_width());

        let third = graphemes.iter().nth(2).ok_or_else(|| eyre!("missing third grapheme"))?;
        merged += third.grapheme();
        assert_eq!(merged.render(&graphemes), "A世👋");
        assert_eq!(
            merged.ascii_width(),
            first.ascii_width() + second.ascii_width() + third.ascii_width()
        );

        Ok(())
    }

    #[test]
    fn test_into_iter_matches_iter_output() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("Hi世👋".to_string());
        let from_iter = graphemes.iter().map(|gt| gt.text).collect::<Vec<_>>();
        let from_into_iter = (&graphemes).into_iter().map(|gt| gt.text).collect::<Vec<_>>();
        assert_eq!(from_iter, from_into_iter);

        Ok(())
    }

    #[test]
    fn test_offsets_are_contiguous_and_cover_content() -> Result<()> {
        let _ = color_eyre::install();

        let text = "Ae\u{301}世🧑‍🚀\nZ".to_string();
        let graphemes = Graphemes::new(text.clone());

        let mut expected_start = 0;
        for segment in &graphemes.graphemes {
            assert_eq!(segment.start(), expected_start);
            assert!(segment.end() >= segment.start());
            expected_start = segment.end();
        }
        assert_eq!(expected_start, text.len());

        Ok(())
    }

    #[test]
    fn test_leading_and_trailing_newline_segments() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("\nA\n".to_string());
        assert_segments(&graphemes, &["\n", "A", "\n"]);

        Ok(())
    }

    #[test]
    fn test_only_newline_segments() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("\n\n\n".to_string());
        assert_segments(&graphemes, &["\n", "\n", "\n"]);

        Ok(())
    }

    #[test]
    fn test_multiple_combining_clusters() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("a\u{0301}o\u{0302}".to_string());
        assert_segments(&graphemes, &["a\u{0301}", "o\u{0302}"]);
        assert_eq!(graphemes[0].ascii_width(), 1);
        assert_eq!(graphemes[1].ascii_width(), 1);

        Ok(())
    }

    #[test]
    fn test_family_emoji_is_single_grapheme() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("👨‍👩‍👧‍👦".to_string());
        assert_segments(&graphemes, &["👨‍👩‍👧‍👦"]);
        assert!(graphemes[0].ascii_width() >= 2);

        Ok(())
    }

    #[test]
    fn test_skin_tone_modifier_is_single_grapheme() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("👍🏽".to_string());
        assert_segments(&graphemes, &["👍🏽"]);
        assert!(graphemes[0].ascii_width() >= 2);

        Ok(())
    }

    #[test]
    fn test_add_assign_keeps_initial_start_and_extends_end() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("ab世".to_string());
        let mut merged = Grapheme::default();

        let mut it = graphemes.iter();
        let first = it.next().ok_or_else(|| eyre!("missing first grapheme"))?;
        let second = it.next().ok_or_else(|| eyre!("missing second grapheme"))?;
        let third = it.next().ok_or_else(|| eyre!("missing third grapheme"))?;

        merged += first.grapheme();
        let initial_start = merged.start();

        merged += second.grapheme();
        assert_eq!(merged.start(), initial_start);
        assert_eq!(merged.render(&graphemes), "ab");

        merged += third.grapheme();
        assert_eq!(merged.start(), initial_start);
        assert_eq!(merged.render(&graphemes), "ab世");

        Ok(())
    }

    #[test]
    fn test_index_out_of_bounds_panics() -> Result<()> {
        let _ = color_eyre::install();

        let graphemes = Graphemes::new("ok".to_string());
        let result = std::panic::catch_unwind(|| {
            let _ = graphemes[2];
        });
        assert!(result.is_err());

        Ok(())
    }
}
