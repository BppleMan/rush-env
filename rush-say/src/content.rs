// use std::ops::Range;
// use unicode_segmentation::UnicodeSegmentation;
// use unicode_width::UnicodeWidthStr;
//
// pub struct TextContent {
//     text: String,
//     graphemes: Vec<TextGrapheme>,
// }
//
// #[derive(Default, Debug, Clone)]
// pub struct TextGrapheme {
//     pub range: Range<usize>,
//     pub width: usize,
// }
//
// #[derive(Default, Debug, Clone)]
// pub struct TextLine {
//     pub graphemes: Vec<TextGrapheme>,
//     pub width: usize,
// }
//
// impl TextContent {
//     pub fn new(text: impl AsRef<str>) -> Self {
//         let text = text.as_ref().to_string();
//         let grapheme_indices = text.grapheme_indices(true);
//         let graphemes = grapheme_indices
//             .map(|(start, grapheme)| {
//                 let range = start..start + grapheme.len();
//                 let width = grapheme.width();
//                 TextGrapheme { range, width }
//             })
//             .collect::<Vec<_>>();
//         Self { text, graphemes }
//     }
//
//     pub fn set_text(&mut self, text: impl AsRef<str>) {
//         let text = text.as_ref().to_string();
//         let grapheme_indices = text.grapheme_indices(true);
//         let graphemes = grapheme_indices
//             .map(|(start, grapheme)| {
//                 let range = start..start + grapheme.len();
//                 let width = grapheme.width();
//                 TextGrapheme { range, width }
//             })
//             .collect::<Vec<_>>();
//         self.text = text;
//         self.graphemes = graphemes;
//     }
//
//     pub fn get_text(&self) -> &str {
//         &self.text
//     }
//
//     pub fn get_graphemes(&self) -> &Vec<TextGrapheme> {
//         &self.graphemes
//     }
// }
//
// impl TextLine {
//     pub fn push(&mut self, grapheme: TextGrapheme) {
//         self.width += grapheme.width;
//         self.graphemes.push(grapheme);
//     }
//
//     pub fn render<'a>(&'a self, content: &'a str) -> &'a str {
//         let range = match (self.graphemes.first(), self.graphemes.last()) {
//             (Some(first), Some(last)) => first.range.start..last.range.end,
//             _ => 0..0, // 如果没有字符，则返回空范围
//         };
//         &content[range]
//     }
// }
