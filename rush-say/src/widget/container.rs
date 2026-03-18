use crate::error::layout_error::{LayoutResult, LayoutResultExt, SiblingIndex, WidgetView};
use crate::layout::{Align, Constraints, Pos, Rect, RectTrait, Size};
use crate::style::BorderStyle;
use crate::widget::Widget;
use crate::widget::logic_box::LogicBox;
use std::io::Write;
use std::ops::{Deref, DerefMut};

#[derive(Default, Debug, Clone)]
pub struct Container<T>
where
    T: Widget,
{
    pub child: T,
    pub state: ContainerState,
}

#[derive(Default, Debug, Clone)]
pub struct ContainerState {
    border: BorderStyle,
    padding: usize,
    margin: usize,
    align: Align,
    size: Size,
    logic_box: LogicBox,
}

impl<T> Deref for Container<T>
where
    T: Widget,
{
    type Target = ContainerState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T> DerefMut for Container<T>
where
    T: Widget,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RowType {
    TopBorder,
    Inner(usize),
    BottomBorder,
}

impl RowType {
    fn detect_row<T>(container: &Container<T>, row: usize) -> RowType
    where
        T: Widget,
    {
        match row {
            r if r < container.top_space() => RowType::TopBorder,
            r if r >= container.size.h().saturating_sub(container.bottom_space()) => RowType::BottomBorder,
            r => RowType::Inner(r),
        }
    }
}

impl Container<()> {
    pub fn empty() -> Self {
        Self::new(())
    }
}

impl<T> Container<T>
where
    T: Widget,
{
    pub fn new(child: T) -> Self {
        Self {
            child,
            state: ContainerState::default(),
        }
    }

    pub fn set_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn set_padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub fn set_margin(mut self, margin: usize) -> Self {
        self.margin = margin;
        self
    }

    pub fn set_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

impl<T> Container<T>
where
    T: Widget,
{
    /// 左空间宽度
    fn left_space(&self) -> usize {
        self.margin + self.border.size + self.padding
    }

    /// 右空间宽度
    fn right_space(&self) -> usize {
        self.padding + self.border.size + self.margin
    }

    /// 上空间宽度
    fn top_space(&self) -> usize {
        self.border.size
    }

    /// 下空间宽度
    fn bottom_space(&self) -> usize {
        self.border.size
    }
}

impl<T> Container<T>
where
    T: Widget,
{
    fn render_left_margin(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if self.margin > 0 {
            write!(writer, "{}", " ".repeat(self.margin))?;
        }
        Ok(())
    }

    fn render_right_margin(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if self.margin > 0 {
            write!(writer, "{}", " ".repeat(self.margin))?;
        }
        Ok(())
    }

    fn render_left_border(&self, writer: &mut impl Write, row: RowType) -> std::io::Result<()> {
        match row {
            RowType::TopBorder => write!(writer, "{}", self.border.top_left),
            RowType::Inner(_) => write!(writer, "{}", self.border.vertical),
            RowType::BottomBorder => write!(writer, "{}", self.border.bottom_left),
        }
    }

    fn render_right_border(&self, writer: &mut impl Write, row: RowType) -> std::io::Result<()> {
        match row {
            RowType::TopBorder => write!(writer, "{}", self.border.top_right),
            RowType::Inner(_) => write!(writer, "{}", self.border.vertical),
            RowType::BottomBorder => write!(writer, "{}", self.border.bottom_right),
        }
    }

    fn render_left_padding(&self, writer: &mut impl Write, row: RowType) -> std::io::Result<()> {
        let horizontal = self.border.horizontal.to_string();
        match row {
            RowType::TopBorder => write!(writer, "{}", horizontal.repeat(self.padding)),
            RowType::Inner(_) => write!(writer, "{}", " ".repeat(self.padding)),
            RowType::BottomBorder => write!(writer, "{}", horizontal.repeat(self.padding)),
        }
    }

    fn render_right_padding(&self, writer: &mut impl Write, row: RowType) -> std::io::Result<()> {
        let horizontal = self.border.horizontal.to_string();
        match row {
            RowType::TopBorder => write!(writer, "{}", horizontal.repeat(self.padding)),
            RowType::Inner(_) => write!(writer, "{}", " ".repeat(self.padding)),
            RowType::BottomBorder => write!(writer, "{}", horizontal.repeat(self.padding)),
        }
    }

    fn create_logic_box(&mut self) {
        let logic_size = Size::new(
            self.size.w().saturating_sub(self.left_space() + self.right_space()),
            self.size.h().saturating_sub(self.top_space() + self.bottom_space()),
        );
        let logic_pos = Pos::default();
        let logic_rect = Rect::pos_size(logic_pos, logic_size);
        let mut logic_box = LogicBox::new(logic_rect, self.align);
        logic_box.set_child_size(self.child.size());
        self.logic_box = logic_box;
    }

    fn render_box(&self, writer: &mut impl Write, row_type: RowType) -> std::io::Result<()> {
        match row_type {
            RowType::TopBorder => write!(writer, "{}", self.border.horizontal.to_string().repeat(self.logic_box.rect.w()))?,
            RowType::Inner(row) => self.logic_box.render(writer, &self.child, row - self.top_space())?,
            RowType::BottomBorder => write!(writer, "{}", self.border.horizontal.to_string().repeat(self.logic_box.rect.w()))?,
        };
        Ok(())
    }
}

impl<T> Widget for Container<T>
where
    T: Widget,
{
    fn name(&self) -> &'static str {
        "Container"
    }

    fn size(&self) -> Size {
        self.size
    }

    fn layout(&mut self, constraints: Constraints) -> LayoutResult {
        let left_space = self.left_space();
        let right_space = self.right_space();
        let child_max_width = constraints.max_width.saturating_sub(left_space + right_space);
        let parent_frame = self.layout_frame(constraints);
        self.child
            .layout(Constraints {
                max_width: child_max_width,
                ..Default::default()
            })
            .with_parent(parent_frame, SiblingIndex::Only)?;
        let max_width = child_max_width + left_space + right_space;
        let max_height = self.child.size().h() + self.top_space() + self.bottom_space();
        self.size = Size::new(max_width, max_height);
        self.create_logic_box();
        // println!("Container: {:?}", self.size);
        // println!("Container: {:?}", self.logic_box);
        Ok(())
    }

    fn render(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        let row_type = RowType::detect_row(self, row);
        self.render_left_margin(writer)?;
        self.render_left_border(writer, row_type)?;
        self.render_left_padding(writer, row_type)?;
        self.render_box(writer, row_type)?;
        self.render_right_padding(writer, row_type)?;
        self.render_right_border(writer, row_type)?;
        self.render_right_margin(writer)?;
        Ok(())
    }

    fn widget_view(&self) -> WidgetView {
        WidgetView::new(self.name())
            .with_align(self.align)
            .with_margin(self.margin)
            .with_border(self.border)
            .with_padding(self.padding)
    }
}

#[cfg(test)]
mod tests {
    use super::Container;
    use crate::layout::{Align, Constraints};
    use crate::style::BorderStyle;
    use crate::widget::{Text, Widget};
    use color_eyre::{Result, eyre::eyre};
    use insta::assert_snapshot;
    use std::io::Cursor;

    #[test]
    fn test_container_layout_and_render_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut container = Container::empty().set_border(BorderStyle::single());
        container.layout(Constraints::new(10, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        container.render(&mut row_0, 0)?;
        let mut row_1 = Cursor::new(vec![]);
        container.render(&mut row_1, 1)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|",
                container.size().w(),
                container.size().h(),
                String::from_utf8(row_0.into_inner())?,
                String::from_utf8(row_1.into_inner())?,
            ),
            @r"
            size: 10x2
            row_0: |┌────────┐|
            row_1: |└────────┘|
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

        let mut container = Container::new(Text::new("ab").set_align(Align::Left))
            .set_border(BorderStyle::single())
            .set_padding(1)
            .set_align(Align::Left);
        container.layout(Constraints::new(10, 0))?;
        let mut row = Cursor::new(vec![]);
        container.render(&mut row, 1)?;
        assert_snapshot!(
            String::from_utf8(row.into_inner())?,
            @r###"│ ab     │"###
        );

        let mut container = Container::new(Text::new("ab").set_align(Align::Left))
            .set_border(BorderStyle::single())
            .set_padding(1)
            .set_align(Align::Center);
        container.layout(Constraints::new(10, 0))?;
        let mut row = Cursor::new(vec![]);
        container.render(&mut row, 1)?;
        assert_snapshot!(
            String::from_utf8(row.into_inner())?,
            @r###"│   ab   │"###
        );

        let mut container = Container::new(Text::new("ab").set_align(Align::Left))
            .set_border(BorderStyle::single())
            .set_padding(1)
            .set_align(Align::Right);
        container.layout(Constraints::new(10, 0))?;
        let mut row = Cursor::new(vec![]);
        container.render(&mut row, 1)?;
        assert_snapshot!(
            String::from_utf8(row.into_inner())?,
            @r###"│     ab │"###
        );

        let mut container = Container::new(Text::new("abcdef").set_align(Align::Left))
            .set_border(BorderStyle::single())
            .set_padding(1)
            .set_align(Align::Left);
        container.layout(Constraints::new(6, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        container.render(&mut row_0, 0)?;
        let mut row_1 = Cursor::new(vec![]);
        container.render(&mut row_1, 1)?;
        let mut row_2 = Cursor::new(vec![]);
        container.render(&mut row_2, 2)?;
        let mut row_3 = Cursor::new(vec![]);
        container.render(&mut row_3, 3)?;
        let mut row_4 = Cursor::new(vec![]);
        container.render(&mut row_4, 4)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|\nrow_2: |{}|\nrow_3: |{}|\nrow_4: |{}|",
                container.size().w(),
                container.size().h(),
                String::from_utf8(row_0.into_inner())?,
                String::from_utf8(row_1.into_inner())?,
                String::from_utf8(row_2.into_inner())?,
                String::from_utf8(row_3.into_inner())?,
                String::from_utf8(row_4.into_inner())?,
            ),
            @r"
            size: 6x5
            row_0: |┌────┐|
            row_1: |│ ab │|
            row_2: |│ cd │|
            row_3: |│ ef │|
            row_4: |└────┘|
            "
        );

        Ok(())
    }

    #[test]
    fn test_container_layout_error_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut container = Container::new(Text::new("ab").set_align(Align::Left))
            .set_border(BorderStyle::single())
            .set_padding(1)
            .set_align(Align::Left);
        let err = match container.layout(Constraints::new(3, 0)) {
            Err(err) => err,
            Ok(()) => return Err(eyre!("container.layout should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            Container (◁, 0|1|1) { 0 x 0 } |-> 3 <-|
              |
              +-- Text (◁) { 0 x 0 } |-> 0 <-|

            "ab"
             ^
            ASCII-width 为 1
            "#
        );

        let mut container = Container::new(Text::new("🎉").set_align(Align::Left))
            .set_border(BorderStyle::single())
            .set_padding(1)
            .set_align(Align::Left);
        let err = match container.layout(Constraints::new(5, 0)) {
            Err(err) => err,
            Ok(()) => return Err(eyre!("container.layout should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            Container (◁, 0|1|1) { 0 x 0 } |-> 5 <-|
              |
              +-- Text (◁) { 0 x 0 } |-> 1 <-|

            "🎉"
             ^^
            ASCII-width 为 2
            "#
        );

        Ok(())
    }
}
