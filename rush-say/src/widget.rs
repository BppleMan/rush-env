mod text;
mod container;
mod list;
mod logic_box;

use crate::error::layout_error::{LayoutDiagnostic, LayoutError, LayoutFrame, LayoutResult, WidgetView};
use crate::layout::Constraints;
use crate::layout::Size;
pub use container::Container;
pub use list::List;
pub use text::Text;

pub trait Widget {
    fn name(&self) -> &'static str;

    fn size(&self) -> Size;

    fn layout_error(&self, constraints: Constraints, diagnostic: LayoutDiagnostic) -> LayoutError {
        LayoutError::leaf(self.layout_frame(constraints), diagnostic)
    }

    fn layout(&mut self, constraints: Constraints) -> LayoutResult;

    fn render(&self, writer: &mut impl std::io::Write, row: usize) -> std::io::Result<()>;

    fn widget_view(&self) -> WidgetView;

    fn layout_frame(&self, constraints: Constraints) -> LayoutFrame {
        LayoutFrame::new(self.widget_view(), self.size(), constraints)
    }
}

/// 空 widget，零尺寸、不输出任何内容。
/// 用于 `Container::empty()` 等场景，让 Container 只渲染边框而无正文。
impl Widget for () {
    fn name(&self) -> &'static str {
        "()"
    }

    fn size(&self) -> Size {
        Size::default()
    }

    fn layout(&mut self, _constraints: Constraints) -> LayoutResult {
        Ok(())
    }

    fn render(&self, _writer: &mut impl std::io::Write, _row: usize) -> std::io::Result<()> {
        Ok(())
    }

    fn widget_view(&self) -> WidgetView {
        WidgetView::new(self.name())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::layout_error::{LayoutDiagnostic, SourceSpan, WidgetView};
    use crate::layout::{Align, Constraints, Size};
    use crate::style::BorderStyle;
    use crate::widget::{Container, Text, Widget};
    use color_eyre::Result;
    use insta::assert_snapshot;
    use std::io::Cursor;

    struct StubWidget {
        size: Size,
    }

    impl Widget for StubWidget {
        fn name(&self) -> &'static str {
            "Stub"
        }

        fn size(&self) -> Size {
            self.size
        }

        fn layout(&mut self, _constraints: Constraints) -> crate::error::layout_error::LayoutResult {
            Ok(())
        }

        fn render(&self, writer: &mut impl std::io::Write, row: usize) -> std::io::Result<()> {
            match row {
                0 => write!(writer, "abc"),
                _ => write!(writer, "   "),
            }
        }

        fn widget_view(&self) -> WidgetView {
            WidgetView::new(self.name()).with_align(Align::Center)
        }
    }

    #[test]
    fn test_widget_render_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut unit = ();
        unit.layout(Constraints::new(10, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        unit.render(&mut row_0, 0)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|",
                unit.size().w(),
                unit.size().h(),
                String::from_utf8(row_0.into_inner())?,
            ),
            @r"
            size: 0x0
            row_0: ||
            "
        );

        let mut text = Text::new("11223344556677889900").set_align(Align::Center);
        text.layout(Constraints::new(10, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        text.render(&mut row_0, 0)?;
        let mut row_1 = Cursor::new(vec![]);
        text.render(&mut row_1, 1)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|",
                text.size().w(),
                text.size().h(),
                String::from_utf8(row_0.into_inner())?,
                String::from_utf8(row_1.into_inner())?,
            ),
            @r"
            size: 10x2
            row_0: |1122334455|
            row_1: |6677889900|
            "
        );

        let text = Text::new("ab").set_align(Align::Left);
        let mut container = Container::new(text)
            .set_align(Align::Center)
            .set_margin(1)
            .set_padding(1)
            .set_border(BorderStyle::single());
        container.layout(Constraints::new(12, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        container.render(&mut row_0, 0)?;
        let mut row_1 = Cursor::new(vec![]);
        container.render(&mut row_1, 1)?;
        let mut row_2 = Cursor::new(vec![]);
        container.render(&mut row_2, 2)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|\nrow_2: |{}|",
                container.size().w(),
                container.size().h(),
                String::from_utf8(row_0.into_inner())?,
                String::from_utf8(row_1.into_inner())?,
                String::from_utf8(row_2.into_inner())?,
            ),
            @r"
            size: 12x3
            row_0: | ┌────────┐ |
            row_1: | │   ab   │ |
            row_2: | └────────┘ |
            "
        );

        Ok(())
    }

    #[test]
    fn test_widget_layout_error_helper_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let widget = StubWidget { size: Size::new(3, 2) };
        let err = widget.layout_error(
            Constraints::new(8, 0),
            LayoutDiagnostic::plain("stub layout failed", "abc", SourceSpan::new(1, 1), "expected demo span"),
        );

        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: stub layout failed

            Stub (⧈) { 3 x 2 } |-> 8 <-|

            abc
             ^
            expected demo span
            "#
        );

        Ok(())
    }
}
