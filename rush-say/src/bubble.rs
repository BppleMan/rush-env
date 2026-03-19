use crate::error::layout_error::{LayoutFrame, LayoutResultExt, SiblingIndex, WidgetView};
use crate::layout::{Align, Axis};
use crate::layout::{Constraints, Size};
use crate::style::{BorderStyle, CommentStyle};
use crate::widget::{Container, List, Text, Widget};
use color_eyre::Result;
use rush_macros::Setter;
use std::io::Write;

#[derive(Setter)]
pub struct Bubble<'a, W>
where
    W: Write,
{
    pub writer: &'a mut W,
    pub text_align: Align,
    pub border: BorderStyle,
    pub padding: usize,
    pub margin: usize,
    pub align: Align,
    pub axis: Axis,
    pub comment: Option<CommentStyle>,
    pub width: usize,
}

impl<'a, W> Bubble<'a, W>
where
    W: Write,
{
    pub fn new(writer: &'a mut W) -> Self {
        Bubble {
            writer,
            text_align: Align::Left,
            border: BorderStyle::rounded(),
            padding: 5,
            margin: 1,
            align: Align::Center,
            axis: Axis::Vertical,
            comment: None,
            width: 40,
        }
    }

    pub fn example(writer: &'a mut W) -> Self {
        Self {
            writer,
            text_align: Align::Left,
            border: BorderStyle::rounded(),
            padding: 4,
            margin: 1,
            align: Align::Center,
            axis: Axis::Vertical,
            comment: Some(CommentStyle::shell()),
            width: 88,
        }
    }

    pub fn shell(writer: &'a mut W) -> Self {
        Self {
            writer,
            text_align: Align::Left,
            border: BorderStyle::rounded(),
            padding: 4,
            margin: 1,
            align: Align::Center,
            axis: Axis::Vertical,
            comment: Some(CommentStyle::shell()),
            width: 40,
        }
    }

    pub fn random_style(&mut self, random: usize) -> &mut Self {
        self.set_border(BorderStyle::random(random))
            .set_comment(Some(CommentStyle::random(random)))
    }
}

impl<'a, W> Bubble<'a, W>
where
    W: Write,
{
    pub fn say(&mut self, text: impl AsRef<str>) -> Result<()> {
        let text = self.build_text(text);
        let mut container = self.build_container(text);
        container
            .layout(Constraints {
                max_width: self.width,
                ..Default::default()
            })
            .with_parent(self.layout_frame(), SiblingIndex::Only)?;
        self.render_widget(&container)?;
        Ok(())
    }

    pub fn say_more<I, T>(&mut self, texts: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let texts = texts.into_iter().map(|t| self.build_text(t)).collect::<Vec<_>>();
        let list = self.build_list(texts);
        let mut container = self.build_container(list);
        container
            .layout(Constraints {
                max_width: self.width,
                ..Default::default()
            })
            .with_parent(self.layout_frame(), SiblingIndex::Only)?;
        self.render_widget(&container)?;
        Ok(())
    }

    fn build_container<T>(&self, widget: T) -> Container<T>
    where
        T: Widget,
    {
        Container::new(widget)
            .set_border(self.border)
            .set_padding(self.padding)
            .set_margin(self.margin)
            .set_align(self.align)
    }

    fn build_list<T>(&self, widgets: Vec<T>) -> List<T>
    where
        T: Widget,
    {
        List::new(widgets).set_axis(self.axis).set_align(self.align).set_border(self.border)
    }

    fn build_text(&self, text: impl AsRef<str>) -> Text {
        Text::new(text).set_align(self.text_align)
    }

    fn render_widget(&mut self, widget: &impl Widget) -> std::io::Result<()> {
        if let Some(comment_open) = self.comment.as_ref().and_then(|c| c.open) {
            writeln!(self.writer, "{comment_open}")?;
        }
        let h = widget.size().h();
        for row_idx in 0..h {
            if let Some(comment_prefix) = self.comment.as_ref().map(|c| c.prefix) {
                write!(self.writer, "{comment_prefix}")?;
            }
            widget.render(self.writer, row_idx)?;
            if let Some(comment_suffix) = self.comment.as_ref().and_then(|c| c.suffix) {
                write!(self.writer, "{comment_suffix}")?;
            }
            writeln!(self.writer)?;
        }
        if let Some(comment_close) = self.comment.as_ref().and_then(|c| c.close) {
            writeln!(self.writer, "{comment_close}")?;
        }
        Ok(())
    }
}

impl<'a, W> Bubble<'a, W>
where
    W: Write,
{
    fn name(&self) -> &'static str {
        "Bubble"
    }

    fn size(&self) -> Size {
        Size::new(self.width, 0)
    }

    fn widget_view(&self) -> WidgetView {
        WidgetView::new(self.name())
            .with_align(self.align)
            .with_axis(self.axis)
            .with_margin(self.margin)
            .with_border(self.border)
            .with_padding(self.padding)
    }

    fn layout_frame(&self) -> LayoutFrame {
        LayoutFrame::new(self.widget_view(), self.size(), Constraints::new(self.width, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::Bubble;
    use crate::layout::{Align, Axis};
    use crate::style::{BorderStyle, CommentStyle};
    use color_eyre::{Result, eyre::eyre};
    use insta::assert_snapshot;

    #[test]
    fn test_bubble_render_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut buffer = Vec::new();
        let mut bubble = Bubble::new(&mut buffer);
        bubble
            .set_width(20)
            .set_padding(0)
            .set_margin(0)
            .set_align(Align::Left)
            .set_text_align(Align::Left)
            .set_border(BorderStyle::single())
            .set_comment(Some(CommentStyle::java_doc()));
        bubble.say("hi")?;
        assert_snapshot!(
            format!(
                "|{}|",
                String::from_utf8(buffer)?
                    .trim_end_matches('\n')
                    .replace('\n', "|\n|")
            ),
            @r"
            |/**|
            | * ┌──────────────────┐|
            | * │hi                │|
            | * └──────────────────┘|
            | */|
            "
        );

        let mut buffer = Vec::new();
        let mut bubble = Bubble::new(&mut buffer);
        bubble
            .set_width(20)
            .set_padding(2)
            .set_margin(1)
            .set_border(BorderStyle::single())
            .set_align(Align::Left)
            .set_text_align(Align::Right);
        bubble.say("hi\na")?;
        assert_snapshot!(
            format!(
                "|{}|",
                String::from_utf8(buffer)?
                    .trim_end_matches('\n')
                    .replace('\n', "|\n|")
            ),
            @r"
            | ┌────────────────┐ |
            | │  hi            │ |
            | │   a            │ |
            | └────────────────┘ |
            "
        );

        let mut buffer = Vec::new();
        let mut bubble = Bubble::new(&mut buffer);
        bubble
            .set_axis(Axis::Horizontal)
            .set_width(40)
            .set_padding(2)
            .set_margin(1)
            .set_border(BorderStyle::rounded())
            .set_align(Align::Center)
            .set_text_align(Align::Center);
        bubble.say_more(["Hello", "World", "你好世界"])?;
        assert_snapshot!(
            format!(
                "|{}|",
                String::from_utf8(buffer)?
                    .trim_end_matches('\n')
                    .replace('\n', "|\n|")
            ),
            @r"
            | ╭────────────────────────────────────╮ |
            | │        Hello │World │你好世界        │ |
            | ╰────────────────────────────────────╯ |
            "
        );

        Ok(())
    }

    #[test]
    fn test_bubble_layout_error_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut buffer = Vec::new();
        let mut bubble = Bubble::new(&mut buffer);
        bubble
            .set_width(1)
            .set_padding(0)
            .set_margin(0)
            .set_align(Align::Left)
            .set_text_align(Align::Left)
            .set_border(BorderStyle::single());
        let err = match bubble.say("🎉") {
            Err(err) => err,
            Ok(()) => return Err(eyre!("bubble.say should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            Bubble (◁, ↕, 0|1|0) { 1 x 0 } |-> 1 <-|
              |
              +-- Container (◁, 0|1|0) { 0 x 0 } |-> 1 <-|
                  |
                  +-- Text (◁) { 0 x 0 } |-> 0 <-|

            "🎉"
             ^^
            ASCII-width 为 2
            "#
        );

        let mut buffer = Vec::new();
        let mut bubble = Bubble::new(&mut buffer);
        bubble
            .set_axis(Axis::Horizontal)
            .set_width(3)
            .set_padding(0)
            .set_margin(0)
            .set_align(Align::Left)
            .set_text_align(Align::Left)
            .set_border(BorderStyle::single());
        let err = match bubble.say_more(["", "🎉"]) {
            Err(err) => err,
            Ok(()) => return Err(eyre!("bubble.say_more should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            Bubble (◁, ↔, 0|1|0) { 3 x 0 } |-> 3 <-|
              |
              +-- Container (◁, 0|1|0) { 0 x 0 } |-> 3 <-|
                  |
                  +-- List (◁, ↔) { 0 x 0 } |-> 1 <-| divide_size=1
                      |
                      +-- ...
                      |
                      +-- [1] Text (◁) { 0 x 0 } |-> 0 <-|

            "🎉"
             ^^
            ASCII-width 为 2
            "#
        );

        Ok(())
    }
}
