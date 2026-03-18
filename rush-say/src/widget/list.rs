use crate::error::layout_error::{LayoutResult, LayoutResultExt, SiblingIndex, WidgetView};
use crate::layout::{Align, Axis, Constraints, Pos, Rect, RectTrait, Size};
use crate::style::BorderStyle;
use crate::widget::Widget;
use crate::widget::logic_box::LogicBox;
use std::io::Write;
use std::ops::{Deref, DerefMut};

#[derive(Default, Debug, Clone)]
pub struct List<T>
where
    T: Widget,
{
    pub children: Vec<T>,
    pub state: ListState,
}

#[derive(Default, Debug, Clone)]
pub struct ListState {
    axis: Axis,
    align: Align,
    border: BorderStyle,
    size: Size,
    divide_count: usize,
    divide_size: usize,
    children_box: Vec<LogicBox>,
    divider_pos: Vec<Pos>,
}

impl<T> Deref for List<T>
where
    T: Widget,
{
    type Target = ListState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T> DerefMut for List<T>
where
    T: Widget,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<T> List<T>
where
    T: Widget,
{
    pub fn new(children: Vec<T>) -> Self {
        Self {
            children,
            state: ListState::default(),
        }
    }

    pub fn set_axis(mut self, axis: Axis) -> Self {
        self.state.axis = axis;
        self
    }

    pub fn set_align(mut self, align: Align) -> Self {
        self.state.align = align;
        self
    }

    pub fn set_border(mut self, border_style: BorderStyle) -> Self {
        self.state.border = border_style;
        self
    }

    pub fn axis(&self) -> &Axis {
        &self.state.axis
    }

    pub fn align(&self) -> &Align {
        &self.state.align
    }
}

impl<T> List<T>
where
    T: Widget,
{
    fn detect_divide(&mut self) {
        self.divide_count = self.children.len().saturating_sub(1);
        self.divide_size = match self.axis {
            Axis::Vertical => self.divide_count,
            Axis::Horizontal => self.divide_count * self.border.size,
        }
    }

    fn detect_size(&mut self, max_width: usize, max_height: usize) {
        self.size = match self.axis {
            Axis::Vertical => Size::new(max_width, max_height + self.divide_size),
            Axis::Horizontal => Size::new(max_width + self.divide_size, max_height),
        };
    }

    fn create_logic_box(&mut self) {
        let children_count = self.children.len();
        let mut offset_x = 0;
        let mut offset_y = 0;
        self.children_box.clear();
        self.divider_pos.clear();
        for (i, child) in self.children.iter().enumerate() {
            let child_rect = match self.axis {
                Axis::Vertical => {
                    let size = Size::new(self.size.w(), child.size().h());
                    let pos = self.size.top_center_of(&size);
                    Rect::pos_size(pos, size)
                }
                Axis::Horizontal => {
                    let size = Size::new(self.size.w().saturating_sub(self.divide_size) / children_count, self.size.h());
                    let pos = self.size.left_of(&size);
                    Rect::pos_size(pos, size)
                }
            };
            let mut child_box = LogicBox::new(child_rect, self.align);
            child_box.offset(offset_x, offset_y);
            child_box.layout(child.size());

            match self.axis {
                Axis::Vertical => offset_y += child_box.rect.h(),
                Axis::Horizontal => offset_x += child_box.rect.w(),
            }

            if i < children_count - 1 {
                self.state.divider_pos.push(Pos::new(offset_x, offset_y));
                match self.axis {
                    Axis::Vertical => offset_y += 1,
                    Axis::Horizontal => offset_x += self.border.size,
                }
            }

            self.state.children_box.push(child_box);
        }
    }

    fn render_divider(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self.axis {
            Axis::Vertical => write!(writer, "{}", self.border.horizontal.to_string().repeat(self.size.w())),
            Axis::Horizontal => write!(writer, "{}", self.border.vertical),
        }
    }

    fn render_box(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        for index in 0..self.children_box.len() {
            let child_box = &self.children_box[index];
            let child = &self.children[index];
            let divider = self.divider_pos.get(index);
            match self.axis {
                Axis::Vertical => {
                    if child_box.rect.inside(&Pos::new(0, row)) {
                        child_box.render(writer, child, child_box.rect.relative_row(row))?
                    } else if let Some(pos) = divider
                        && pos.y() == row
                    {
                        self.render_divider(writer)?
                    }
                }
                Axis::Horizontal => {
                    child_box.render(writer, child, row)?;
                    if divider.is_some() {
                        self.render_divider(writer)?
                    }
                }
            };
        }
        Ok(())
    }
}

impl<T> Widget for List<T>
where
    T: Widget,
{
    fn name(&self) -> &'static str {
        "List"
    }

    fn size(&self) -> Size {
        self.size
    }

    fn layout(&mut self, constraints: Constraints) -> LayoutResult {
        self.detect_divide();

        let parent_frame = self.layout_frame(constraints);
        let children_count = self.children.len();
        let child_max_width = match self.axis {
            Axis::Vertical => constraints.max_width,
            Axis::Horizontal => constraints.max_width.saturating_sub(self.divide_size) / self.children.len(),
        };
        let mut max_width = 0;
        let mut max_height = 0;
        for (i, child) in self.children.iter_mut().enumerate() {
            child
                .layout(Constraints {
                    max_width: child_max_width,
                    max_height: Default::default(),
                })
                .with_parent(
                    parent_frame.clone(),
                    SiblingIndex::Index {
                        index: i,
                        total: children_count,
                    },
                )?;
            match self.state.axis {
                Axis::Vertical => {
                    max_width = max_width.max(child.size().w());
                    max_height += child.size().h();
                }
                Axis::Horizontal => {
                    max_width += child.size().w();
                    max_height = max_height.max(child.size().h());
                }
            };
        }
        self.detect_size(max_width, max_height);
        self.create_logic_box();

        Ok(())
    }

    fn render(&self, writer: &mut impl Write, row: usize) -> std::io::Result<()> {
        self.render_box(writer, row)?;
        Ok(())
    }

    fn widget_view(&self) -> WidgetView {
        WidgetView::new(self.name())
            .with_axis(self.axis)
            .with_align(self.align)
            .with_attr("divide_size", self.divide_size)
    }
}

#[cfg(test)]
mod tests {
    use super::List;
    use crate::layout::{Align, Axis, Constraints};
    use crate::style::BorderStyle;
    use crate::widget::{Container, Text, Widget};
    use color_eyre::{Result, eyre::eyre};
    use insta::assert_snapshot;
    use std::io::Cursor;

    #[test]
    fn test_list_layout_and_render_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut list = List::new(vec![Text::new("a"), Text::new("bbb"), Text::new("cc")])
            .set_align(Align::Right)
            .set_border(BorderStyle::double());
        list.layout(Constraints::new(10, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        list.render(&mut row_0, 0)?;
        let mut row_1 = Cursor::new(vec![]);
        list.render(&mut row_1, 1)?;
        let mut row_2 = Cursor::new(vec![]);
        list.render(&mut row_2, 2)?;
        let mut row_3 = Cursor::new(vec![]);
        list.render(&mut row_3, 3)?;
        let mut row_4 = Cursor::new(vec![]);
        list.render(&mut row_4, 4)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|\nrow_2: |{}|\nrow_3: |{}|\nrow_4: |{}|",
                list.size().w(),
                list.size().h(),
                String::from_utf8(row_0.into_inner())?,
                String::from_utf8(row_1.into_inner())?,
                String::from_utf8(row_2.into_inner())?,
                String::from_utf8(row_3.into_inner())?,
                String::from_utf8(row_4.into_inner())?,
            ),
            @r"
            size: 3x5
            row_0: |  a|
            row_1: |═══|
            row_2: |bbb|
            row_3: |═══|
            row_4: | cc|
            "
        );

        let mut list = List::new(vec![
            Container::new(Text::new("A")).set_border(BorderStyle::single()).set_padding(1),
            Container::new(Text::new("1\n2\n3"))
                .set_border(BorderStyle::single())
                .set_padding(1),
            Container::new(Text::new("xy")).set_border(BorderStyle::single()).set_padding(1),
        ])
        .set_axis(Axis::Horizontal)
        .set_align(Align::BottomCenter)
        .set_border(BorderStyle::double());
        list.layout(Constraints::new(20, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        list.render(&mut row_0, 0)?;
        let mut row_1 = Cursor::new(vec![]);
        list.render(&mut row_1, 1)?;
        let mut row_2 = Cursor::new(vec![]);
        list.render(&mut row_2, 2)?;
        let mut row_3 = Cursor::new(vec![]);
        list.render(&mut row_3, 3)?;
        let mut row_4 = Cursor::new(vec![]);
        list.render(&mut row_4, 4)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|\nrow_1: |{}|\nrow_2: |{}|\nrow_3: |{}|\nrow_4: |{}|",
                list.size().w(),
                list.size().h(),
                String::from_utf8(row_0.into_inner())?,
                String::from_utf8(row_1.into_inner())?,
                String::from_utf8(row_2.into_inner())?,
                String::from_utf8(row_3.into_inner())?,
                String::from_utf8(row_4.into_inner())?,
            ),
            @r"
            size: 20x5
            row_0: |      ║┌────┐║      |
            row_1: |      ║│ 1  │║      |
            row_2: |┌────┐║│ 2  │║┌────┐|
            row_3: |│ A  │║│ 3  │║│ xy │|
            row_4: |└────┘║└────┘║└────┘|
            "
        );

        let mut list: List<Text> = List::new(vec![]);
        list.layout(Constraints::new(12, 0))?;
        let mut row_0 = Cursor::new(vec![]);
        list.render(&mut row_0, 0)?;
        assert_snapshot!(
            format!(
                "size: {}x{}\nrow_0: |{}|",
                list.size().w(),
                list.size().h(),
                String::from_utf8(row_0.into_inner())?,
            ),
            @r"
            size: 0x0
            row_0: ||
            "
        );

        Ok(())
    }

    #[test]
    fn test_list_layout_error_cases_snapshot() -> Result<()> {
        let _ = color_eyre::install();

        let mut list = List::new(vec![Text::new(""), Text::new("🎉")]).set_axis(Axis::Horizontal);
        let err = match list.layout(Constraints::new(1, 0)) {
            Err(err) => err,
            Ok(()) => return Err(eyre!("list.layout should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            List (⧈, ↔) { 0 x 0 } |-> 1 <-| divide_size=1
              |
              +-- ...
              |
              +-- [1] Text (⧈) { 0 x 0 } |-> 0 <-|

            "🎉"
             ^^
            ASCII-width 为 2
            "#
        );

        let mut list = List::new(vec![Text::new("🎉"), Text::new("")]).set_axis(Axis::Horizontal);
        let err = match list.layout(Constraints::new(1, 0)) {
            Err(err) => err,
            Ok(()) => return Err(eyre!("list.layout should fail")),
        };
        assert_snapshot!(
            err.to_string(),
            @r#"
            LayoutError: 最大可用宽度无法满足最大字素的 ASCII-width

            List (⧈, ↔) { 0 x 0 } |-> 1 <-| divide_size=1
              |
              +-- [0] Text (⧈) { 0 x 0 } |-> 0 <-|
              |
              +-- ...

            "🎉"
             ^^
            ASCII-width 为 2
            "#
        );

        Ok(())
    }
}
