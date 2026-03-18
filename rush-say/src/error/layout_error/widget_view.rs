use crate::{
    layout::{Align, Axis},
    style::BorderStyle,
};
use std::fmt::{self, Display};

#[derive(Default, Debug, Clone)]
pub struct WidgetView {
    name: &'static str,
    align: Option<Align>,
    axis: Option<Axis>,
    margin: Option<usize>,
    padding: Option<usize>,
    border: Option<BorderStyle>,
    attrs: Vec<WidgetAttr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetAttr {
    name: &'static str,
    value: String,
}

impl WidgetView {
    pub fn new(name: &'static str) -> Self {
        Self { name, ..Self::default() }
    }

    pub fn with_align(mut self, align: Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn with_axis(mut self, axis: Axis) -> Self {
        self.axis = Some(axis);
        self
    }

    pub fn with_margin(mut self, margin: usize) -> Self {
        self.margin = Some(margin);
        self
    }

    pub fn with_padding(mut self, padding: usize) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = Some(border);
        self
    }

    pub fn with_attr(mut self, name: &'static str, value: impl Display) -> Self {
        self.push_attr(name, value);
        self
    }

    pub fn push_attr(&mut self, name: &'static str, value: impl Display) {
        self.attrs.push(WidgetAttr {
            name,
            value: value.to_string(),
        });
    }

    pub fn write_attrs(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for attr in &self.attrs {
            write!(f, " {}={}", attr.name, attr.value)?;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.align.is_none()
            && self.axis.is_none()
            && self.margin.is_none()
            && self.padding.is_none()
            && self.border.is_none()
            && self.attrs.is_empty()
    }
}

impl Display for WidgetView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        let mut parts = Vec::new();

        if let Some(align) = self.align {
            parts.push(align_icon(align).to_string());
        }

        if let Some(axis) = self.axis {
            parts.push(axis_icon(axis).to_string());
        }

        if let Some(box_visual) = box_visual(self.margin, self.border, self.padding) {
            parts.push(box_visual);
        }

        if !parts.is_empty() {
            write!(f, " (")?;
            for (index, part) in parts.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{part}")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

fn axis_icon(axis: Axis) -> &'static str {
    match axis {
        Axis::Horizontal => "↔",
        Axis::Vertical => "↕",
    }
}

fn align_icon(align: Align) -> &'static str {
    match align {
        Align::TopLeft => "◰",
        Align::TopCenter => "△",
        Align::TopRight => "◳",
        Align::Left => "◁",
        Align::Center => "⧈",
        Align::Right => "▷",
        Align::BottomLeft => "◲",
        Align::BottomCenter => "▽",
        Align::BottomRight => "◱",
    }
}

fn box_visual(margin: Option<usize>, border: Option<BorderStyle>, padding: Option<usize>) -> Option<String> {
    if margin.is_none() && border.is_none() && padding.is_none() {
        None
    } else {
        Some(format!(
            "{}|{}|{}",
            margin.unwrap_or(0),
            border.map(|border| border.size).unwrap_or(0),
            padding.unwrap_or(0)
        ))
    }
}
