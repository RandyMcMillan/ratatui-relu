//! A 3×3 grid widget that reports the dimensions of neighboring panels.
//!
//! Designed to be used inside the main area of [`MultiPanel`](crate::multi_panel::MultiPanel).

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::multi_panel::MultiPanelState;

/// Renders a 3×3 grid of sub-widgets, each reporting a neighbor's size.
#[derive(Debug, Clone)]
pub struct NeighborReportGrid {
    left_fraction: f64,
    right_fraction: f64,
    header_height: u16,
    footer_height: u16,
    center_content: Option<String>,
}

impl NeighborReportGrid {
    /// Create a report grid with the same layout fractions used by the outer
    /// [`MultiPanel`](crate::multi_panel::MultiPanel).
    pub fn new(
        left_fraction: f64,
        right_fraction: f64,
        header_height: u16,
        footer_height: u16,
    ) -> Self {
        Self {
            left_fraction,
            right_fraction,
            header_height,
            footer_height,
            center_content: None,
        }
    }

    /// Override the center cell's content. If not set, the center cell shows
    /// the main area dimensions.
    pub fn center_content(mut self, content: impl Into<String>) -> Self {
        self.center_content = Some(content.into());
        self
    }
}

impl StatefulWidget for NeighborReportGrid {
    type State = MultiPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(area);

        let cols_top = split_three(rows[0]);
        let cols_mid = split_three(rows[1]);
        let cols_bot = split_three(rows[2]);

        subwidget(
            "Top-Left",
            format!("{}x{}", cols_top[0].width, cols_top[0].height),
            cols_top[0],
            buf,
        );
        subwidget(
            "Top",
            format!("{} rows", state.header_height(self.header_height)),
            cols_top[1],
            buf,
        );
        subwidget(
            "Top-Right",
            format!("{}x{}", cols_top[2].width, cols_top[2].height),
            cols_top[2],
            buf,
        );

        subwidget(
            "Left",
            format!("{} cols", state.left_sidebar_width(area.width, self.left_fraction)),
            cols_mid[0],
            buf,
        );
        subwidget(
            "Center",
            self.center_content
                .clone()
                .unwrap_or_else(|| format!("{}x{}", area.width, area.height)),
            cols_mid[1],
            buf,
        );
        subwidget(
            "Right",
            format!("{} cols", state.right_sidebar_width(area.width, self.right_fraction)),
            cols_mid[2],
            buf,
        );

        subwidget(
            "Bot-Left",
            format!("{}x{}", cols_bot[0].width, cols_bot[0].height),
            cols_bot[0],
            buf,
        );
        subwidget(
            "Bottom",
            format!("{} rows", state.footer_height(self.footer_height)),
            cols_bot[1],
            buf,
        );
        subwidget(
            "Bot-Right",
            format!("{}x{}", cols_bot[2].width, cols_bot[2].height),
            cols_bot[2],
            buf,
        );
    }
}

fn split_three(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
        .split(area)
}

fn subwidget(title: &str, value: String, area: Rect, buf: &mut Buffer) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default());
    let inner = block.inner(area);
    Widget::render(block, area, buf);
    if inner.width > 0 && inner.height > 0 {
        let p = Paragraph::new(Text::from(value)).alignment(Alignment::Center);
        Widget::render(p, inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_grid_renders_without_panic() {
        let mut state = MultiPanelState::open(4.0);
        let widget = NeighborReportGrid::new(0.25, 0.25, 5, 5);
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        StatefulWidget::render(widget, buf.area, &mut buf, &mut state);
    }
}
