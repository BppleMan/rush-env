use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::error::layout_error::{LayoutDiagnostic, LayoutResult, WidgetView};
use crate::layout::{Align, Constraints, Size};
use crate::model::{Grapheme, GraphemeText, Graphemes};
use crate::widget::Widget;
use rush_macros::FieldName;

#[derive(Default, Debug, Clone, FieldName)]
pub struct Text {
    pub content: Graphemes,
    pub align: Align,
    state: TextState,
    size: Size,
}

#[derive(Default, Debug, Clone, FieldName)]
pub struct TextState {
    lines: Vec<Grapheme>,
}

impl Deref for Text {
    type Target = TextState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Text {
    pub fn new(content: impl AsRef<str>) -> Text {
        Self::with_align(content, Align::Center)
    }

    pub fn with_align(content: impl AsRef<str>, align: Align) -> Text {
        let content = Graphemes::new(content.as_ref().to_string());
        Text {
            content,
            align,
            ..Default::default()
        }
    }

    pub fn set_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

impl Text {
    fn detect_max_width(&self, max_width: usize) -> Option<(usize, GraphemeText<'_>)> {
        self.content.iter().map(|g| (g.ascii_width(), g)).find(|(w, _)| *w > max_width)
    }

    fn wrap_text(&self, constraints: Constraints) -> LayoutResult<Vec<Grapheme>> {
        if let Some((max_grapheme_width, grapheme)) = self.detect_max_width(constraints.max_width)
            && constraints.max_width < max_grapheme_width
        {
            return Err(self.layout_error(
                constraints,
                LayoutDiagnostic::quoted(
                    "最大可用宽度无法满足最大字素的 ASCII-width",
                    self.content.get_content(),
                    &grapheme,
                    format!("ASCII-width 为 {}", grapheme.ascii_width()),
                ),
            ));
        }
        let mut wrapped_lines: Vec<Grapheme> = vec![];
        let mut line_buffer: Grapheme = Grapheme::default();
        for grapheme in &self.content {
            match grapheme.as_str() {
                "\n" | "\r" | "\r\n" => {
                    wrapped_lines.push(std::mem::take(&mut line_buffer));
                }
                _ => {
                    if line_buffer.ascii_width() + grapheme.ascii_width() > constraints.max_width {
                        wrapped_lines.push(std::mem::take(&mut line_buffer));
                    }
                    line_buffer += grapheme.grapheme();
                }
            }
        }
        wrapped_lines.push(line_buffer);
        Ok(wrapped_lines)
    }

    /// 越界时输出等宽空格
    fn render_blank(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", " ".repeat(self.size.w()))
    }

    /// 计算某行文本的左右对齐留白
    fn align_padding(&self, line_width: usize) -> (usize, usize) {
        let remaining = self.size.w().saturating_sub(line_width);
        match self.align {
            Align::TopLeft | Align::Left | Align::BottomLeft => (0, remaining),
            Align::TopCenter | Align::Center | Align::BottomCenter => (remaining / 2, remaining - (remaining / 2)),
            Align::TopRight | Align::Right | Align::BottomRight => (remaining, 0),
        }
    }

    /// 渲染一行文本内容（含对齐留白）
    fn render_line(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        let line = &self.state.lines[row];
        let rendered_line = line.render(&self.content);
        let (left, right) = self.align_padding(line.ascii_width());
        write!(writer, "{}{}{}", " ".repeat(left), rendered_line, " ".repeat(right))
    }
}

impl Widget for Text {
    fn name(&self) -> &'static str {
        "Text"
    }

    fn size(&self) -> Size {
        self.size
    }

    fn layout(&mut self, constraints: Constraints) -> LayoutResult {
        self.state.lines = self.wrap_text(constraints)?;
        let width = self.state.lines.iter().map(Grapheme::ascii_width).max().unwrap_or(0);
        let height = self.state.lines.len();
        self.size = Size::new(width, height);
        Ok(())
    }

    fn render(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        if row >= self.size.h() {
            return self.render_blank(writer);
        }
        self.render_line(writer, row)
    }

    fn widget_view(&self) -> WidgetView {
        WidgetView::new(self.name()).with_align(self.align)
    }
}

impl FromStr for Text {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Text::new(s))
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::{Align, Constraints};
    use crate::widget::Widget;
    use crate::widget::text::Text;
    use color_eyre::{Result, eyre::eyre};
    use insta::assert_snapshot;
    use std::io::{Cursor, Write as _};

    fn render_all(w: &impl Widget) -> Result<String> {
        let mut buf = Cursor::new(vec![]);
        let h = w.size().h();
        for row in 0..h {
            w.render(&mut buf, row)?;
            if row + 1 < h {
                writeln!(&mut buf)?;
            }
        }
        Ok(String::from_utf8(buf.into_inner())?)
    }

    fn render_row(w: &impl Widget, row: usize) -> Result<String> {
        let mut buf = Cursor::new(vec![]);
        w.render(&mut buf, row)?;
        Ok(String::from_utf8(buf.into_inner())?)
    }

    #[test]
    fn test_layout_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut text = Text::with_align("ab", Align::Center);
        let err = match text.layout(Constraints {
            max_width: 0,
            ..Default::default()
        }) {
            Err(err) => err,
            Ok(()) => return Err(eyre!("text.layout should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            Text (⧈) { 0 x 0 } |-> 0 <-|

            "ab"
             ^
            ASCII-width 为 1
            "#
        );

        let mut text = Text::with_align("", Align::Center);
        text.layout(Constraints::new(10, 0))?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|",
                text.size().w(),
                text.size().h(),
                render_row(&text, 0)?,
            ),
            @r"
            size: 0x1
            row_0: ||
            "
        );

        let mut text = Text::with_align("hello world", Align::Center);
        text.layout(Constraints::new(6, 0))?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|\nrendered: |{}|",
                text.size().w(),
                text.size().h(),
                render_row(&text, 0)?,
                render_row(&text, 1)?,
                render_all(&text)?.replace('\n', "\\n"),
            ),
            @r"
            size: 6x2
            row_0: |hello |
            row_1: |world |
            rendered: |hello \nworld |
            "
        );

        Ok(())
    }

    #[test]
    fn test_wrap_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let text = Text::new("a\r\n\nb\rc\n");
        let lines = text.wrap_text(Constraints::new(10, 0))?;
        assert_snapshot!(
            format!(
                "line_count: {}\n[0] width={} text=|{}|\n[1] width={} text=|{}|\n[2] width={} text=|{}|\n[3] width={} text=|{}|\n[4] width={} text=|{}|",
                lines.len(),
                lines[0].ascii_width(),
                lines[0].render(&text.content),
                lines[1].ascii_width(),
                lines[1].render(&text.content),
                lines[2].ascii_width(),
                lines[2].render(&text.content),
                lines[3].ascii_width(),
                lines[3].render(&text.content),
                lines[4].ascii_width(),
                lines[4].render(&text.content),
            ),
            @r"
            line_count: 5
            [0] width=1 text=|a|
            [1] width=0 text=||
            [2] width=1 text=|b|
            [3] width=1 text=|c|
            [4] width=0 text=||
            "
        );

        let text = Text::new("a🧑‍🚀bc");
        let lines = text.wrap_text(Constraints::new(4, 0))?;
        assert_snapshot!(
            format!(
                "line_count: {}\n[0] width={} text=|{}|\n[1] width={} text=|{}|",
                lines.len(),
                lines[0].ascii_width(),
                lines[0].render(&text.content),
                lines[1].ascii_width(),
                lines[1].render(&text.content),
            ),
            @r"
            line_count: 2
            [0] width=4 text=|a🧑‍🚀b|
            [1] width=1 text=|c|
            "
        );

        let text = Text::new("🎉a");
        let err = match text.wrap_text(Constraints::new(1, 0)) {
            Err(err) => err,
            Ok(_) => return Err(eyre!("text.wrap_text should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            Text (⧈) { 0 x 0 } |-> 1 <-|

            "🎉a"
             ^^
            ASCII-width 为 2
            "#
        );

        Ok(())
    }

    #[test]
    fn test_render_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut text = Text::new("hi\na").set_align(Align::Left);
        text.layout(Constraints::new(10, 0))?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|",
                text.size().w(),
                text.size().h(),
                render_row(&text, 0)?,
                render_row(&text, 1)?,
            ),
            @r"
            size: 2x2
            row_0: |hi|
            row_1: |a |
            "
        );

        let mut text = Text::new("hi\na").set_align(Align::Center);
        text.layout(Constraints::new(10, 0))?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|",
                text.size().w(),
                text.size().h(),
                render_row(&text, 0)?,
                render_row(&text, 1)?,
            ),
            @r"
            size: 2x2
            row_0: |hi|
            row_1: |a |
            "
        );

        let mut text = Text::new("hi\na").set_align(Align::Right);
        text.layout(Constraints::new(10, 0))?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|",
                text.size().w(),
                text.size().h(),
                render_row(&text, 0)?,
                render_row(&text, 1)?,
            ),
            @r"
            size: 2x2
            row_0: |hi|
            row_1: | a|
            "
        );

        let mut text = Text::new("ab");
        text.layout(Constraints::new(10, 0))?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|",
                text.size().w(),
                text.size().h(),
                render_row(&text, 0)?,
                render_row(&text, 1)?,
            ),
            @r"
            size: 2x1
            row_0: |ab|
            row_1: |  |
            "
        );

        Ok(())
    }
}
