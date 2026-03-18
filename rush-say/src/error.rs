pub mod layout_error;

pub use layout_error::LayoutError;

#[cfg(test)]
mod tests {
    use crate::error::LayoutError;
    use crate::error::layout_error::{LayoutDiagnostic, LayoutFrame, SiblingIndex, SourceSpan, WidgetView};
    use crate::layout::{Align, Axis, Constraints, Size};
    use crate::style::BorderStyle;
    use color_eyre::Result;
    use insta::assert_snapshot;

    fn constraints(max_width: usize) -> Constraints {
        Constraints { max_width, max_height: 0 }
    }

    fn frame(view: WidgetView, width: usize, height: usize, max_width: usize) -> LayoutFrame {
        LayoutFrame::new(view, Size::new(width, height), constraints(max_width))
    }

    #[test]
    fn test_layout_error_renders_focus_tree_and_leaf_snippet() -> Result<()> {
        let _ = color_eyre::install();

        let error = LayoutError::leaf(
            frame(WidgetView::new("Text").with_align(Align::Center), 2, 1, 1),
            LayoutDiagnostic::quoted(
                "宽度约束不满足",
                "🎉a",
                SourceSpan::new(0, "🎉".len()),
                r#"grapheme[0] = "🎉", required width = 2, indivisible"#,
            ),
        )
        .in_parent(
            frame(
                WidgetView::new("List").with_axis(Axis::Horizontal).with_align(Align::Center),
                3,
                1,
                3,
            ),
            SiblingIndex::index(5, 9),
        )
        .in_parent(
            frame(
                WidgetView::new("Container")
                    .with_margin(1)
                    .with_align(Align::Center)
                    .with_border(BorderStyle::single())
                    .with_padding(2),
                3,
                3,
                3,
            ),
            SiblingIndex::Only,
        );

        assert_snapshot!(
            error.to_string(),
            @r#"
            LayoutError: 宽度约束不满足

            Container (⧈, 1|1|2) { 3 x 3 } |-> 3 <-|
              |
              +-- List (⧈, ↔) { 3 x 1 } |-> 3 <-|
                  |
                  +-- ...
                  |
                  +-- [5] Text (⧈) { 2 x 1 } |-> 1 <-|
                  |
                  +-- ...

            "🎉a"
             ^^
            grapheme[0] = "🎉", required width = 2, indivisible
            "#
        );

        Ok(())
    }

    #[test]
    fn test_child_windows_render_expected_elision_patterns() -> Result<()> {
        let _ = color_eyre::install();

        let only = LayoutError::leaf(
            frame(WidgetView::new("Text"), 1, 1, 8),
            LayoutDiagnostic::quoted("宽度约束不满足", "a", SourceSpan::new(0, 1), "required width = 1"),
        )
        .in_parent(
            frame(WidgetView::new("List").with_axis(Axis::Vertical), 1, 1, 8),
            SiblingIndex::Only,
        );

        let first = LayoutError::leaf(
            frame(WidgetView::new("Text"), 2, 1, 2),
            LayoutDiagnostic::quoted("宽度约束不满足", "ab", SourceSpan::new(0, 1), "required width = 2"),
        )
        .in_parent(
            frame(WidgetView::new("List").with_axis(Axis::Horizontal), 8, 1, 8),
            SiblingIndex::index(0, 3),
        );

        let last = LayoutError::leaf(
            frame(WidgetView::new("Text"), 2, 1, 2),
            LayoutDiagnostic::quoted("宽度约束不满足", "ab", SourceSpan::new(0, 1), "required width = 2"),
        )
        .in_parent(
            frame(WidgetView::new("List").with_axis(Axis::Horizontal), 8, 1, 8),
            SiblingIndex::index(9, 10),
        );

        assert_snapshot!(
            only.to_string(),
            @r#"
            LayoutError: 宽度约束不满足

            List (↕) { 1 x 1 } |-> 8 <-|
              |
              +-- Text { 1 x 1 } |-> 8 <-|

            "a"
             ^
            required width = 1
            "#
        );

        assert_snapshot!(
            first.to_string(),
            @r#"
            LayoutError: 宽度约束不满足

            List (↔) { 8 x 1 } |-> 8 <-|
              |
              +-- [0] Text { 2 x 1 } |-> 2 <-|
              |
              +-- ...

            "ab"
             ^
            required width = 2
            "#
        );

        assert_snapshot!(
            last.to_string(),
            @r#"
            LayoutError: 宽度约束不满足

            List (↔) { 8 x 1 } |-> 8 <-|
              |
              +-- ...
              |
              +-- [9] Text { 2 x 1 } |-> 2 <-|

            "ab"
             ^
            required width = 2
            "#
        );

        Ok(())
    }
}
