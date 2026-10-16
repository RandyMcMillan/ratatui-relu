//! A reusable, parameterized multi-panel widget.
//!
//! Renders a layout with header, footer, left sidebar, right sidebar, and main
//! content areas. Each panel's visibility is controlled by a [`Transition`]
//! stored in [`MultiPanelState`].

use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::animation::Transition;

/// Mutable state for [`MultiPanel`].
#[derive(Debug, Clone)]
pub struct MultiPanelState {
    pub header: Transition,
    pub left_sidebar: Transition,
    pub right_sidebar: Transition,
    pub footer: Transition,
}

impl MultiPanelState {
    /// Create a new state with all panels open.
    pub fn open(speed: f64) -> Self {
        Self {
            header: Transition::new(1.0, speed),
            left_sidebar: Transition::new(1.0, speed),
            right_sidebar: Transition::new(1.0, speed),
            footer: Transition::new(1.0, speed),
        }
    }

    /// Create a new state with all panels closed.
    pub fn closed(speed: f64) -> Self {
        Self {
            header: Transition::new(0.0, speed),
            left_sidebar: Transition::new(0.0, speed),
            right_sidebar: Transition::new(0.0, speed),
            footer: Transition::new(0.0, speed),
        }
    }

    /// Advance all panel transitions by `dt`.
    pub fn update(&mut self, dt: Duration) {
        self.header.update(dt);
        self.left_sidebar.update(dt);
        self.right_sidebar.update(dt);
        self.footer.update(dt);
    }

    /// Toggle all panels at once.
    pub fn toggle_all(&mut self) {
        self.header.toggle_between(0.0, 1.0);
        self.left_sidebar.toggle_between(0.0, 1.0);
        self.right_sidebar.toggle_between(0.0, 1.0);
        self.footer.toggle_between(0.0, 1.0);
    }
}

/// Configuration for [`MultiPanel`].
#[derive(Debug, Clone)]
pub struct MultiPanel {
    pub header_title: String,
    pub left_title: String,
    pub right_title: String,
    pub main_title: String,
    pub footer_title: String,
    /// Optional help text rendered inside the footer.
    pub footer_content: Option<String>,
    pub header_color: Color,
    pub left_color: Color,
    pub right_color: Color,
    pub footer_color: Color,
    pub main_color: Color,
    /// Height of the header/footer when fully open, in rows.
    pub panel_height: u16,
    /// Width of each sidebar as a fraction of total width (0.0–1.0).
    pub sidebar_width_fraction: f64,
}

impl MultiPanel {
    /// Create a [`MultiPanel`] with sensible defaults.
    pub fn new() -> Self {
        Self {
            header_title: "Header".to_string(),
            left_title: "Left".to_string(),
            right_title: "Right".to_string(),
            main_title: "Main".to_string(),
            footer_title: "Footer".to_string(),
            footer_content: None,
            header_color: Color::Blue,
            left_color: Color::Green,
            right_color: Color::Magenta,
            footer_color: Color::Yellow,
            main_color: Color::Reset,
            panel_height: 5,
            sidebar_width_fraction: 0.25,
        }
    }

    /// Builder-style setter for header title.
    pub fn header_title(mut self, title: impl Into<String>) -> Self {
        self.header_title = title.into();
        self
    }

    /// Builder-style setter for left sidebar title.
    pub fn left_title(mut self, title: impl Into<String>) -> Self {
        self.left_title = title.into();
        self
    }

    /// Builder-style setter for right sidebar title.
    pub fn right_title(mut self, title: impl Into<String>) -> Self {
        self.right_title = title.into();
        self
    }

    /// Builder-style setter for main panel title.
    pub fn main_title(mut self, title: impl Into<String>) -> Self {
        self.main_title = title.into();
        self
    }

    /// Builder-style setter for footer title.
    pub fn footer_title(mut self, title: impl Into<String>) -> Self {
        self.footer_title = title.into();
        self
    }

    /// Builder-style setter for footer help/content text.
    pub fn footer_content(mut self, content: impl Into<String>) -> Self {
        self.footer_content = Some(content.into());
        self
    }

    /// Builder-style setter for panel height.
    pub fn panel_height(mut self, height: u16) -> Self {
        self.panel_height = height;
        self
    }

    /// Builder-style setter for sidebar width fraction.
    pub fn sidebar_width_fraction(mut self, fraction: f64) -> Self {
        self.sidebar_width_fraction = fraction.clamp(0.0, 1.0);
        self
    }

    /// Builder-style setter for header border color.
    pub fn header_color(mut self, color: Color) -> Self {
        self.header_color = color;
        self
    }

    /// Builder-style setter for left sidebar border color.
    pub fn left_color(mut self, color: Color) -> Self {
        self.left_color = color;
        self
    }

    /// Builder-style setter for right sidebar border color.
    pub fn right_color(mut self, color: Color) -> Self {
        self.right_color = color;
        self
    }

    /// Builder-style setter for footer border color.
    pub fn footer_color(mut self, color: Color) -> Self {
        self.footer_color = color;
        self
    }

    /// Builder-style setter for main panel border color.
    pub fn main_color(mut self, color: Color) -> Self {
        self.main_color = color;
        self
    }
}

impl Default for MultiPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for MultiPanel {
    type State = MultiPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_height = (state.header.current() * self.panel_height as f64).round() as u16;
        let footer_height = (state.footer.current() * self.panel_height as f64).round() as u16;
        let left_width = (state.left_sidebar.current() * area.width as f64 * self.sidebar_width_fraction).round() as u16;
        let right_width = (state.right_sidebar.current() * area.width as f64 * self.sidebar_width_fraction).round() as u16;

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(0),
                Constraint::Length(footer_height),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_width),
                Constraint::Min(0),
                Constraint::Length(right_width),
            ])
            .split(vertical[1]);

        render_panel(buf, &self.header_title, vertical[0], self.header_color);
        render_panel(buf, &self.left_title, horizontal[0], self.left_color);
        render_panel(buf, &self.main_title, horizontal[1], self.main_color);
        render_panel(buf, &self.right_title, horizontal[2], self.right_color);
        render_footer(
            buf,
            &self.footer_title,
            vertical[2],
            self.footer_color,
            self.footer_content.as_deref(),
        );
    }
}

fn render_footer(buf: &mut Buffer, title: &str, area: Rect, color: Color, content: Option<&str>) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    Widget::render(block, area, buf);

    if inner.width > 0 && inner.height > 0 {
        let fallback = format!("{} area", title);
        let text = content.unwrap_or(&fallback);
        let paragraph = Paragraph::new(Text::from(text)).alignment(Alignment::Center);
        Widget::render(paragraph, inner, buf);
    }
}

fn render_panel(buf: &mut Buffer, title: &str, area: Rect, color: Color) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    Widget::render(block, area, buf);

    if inner.width > 0 && inner.height > 0 {
        let text = Paragraph::new(Text::from(format!("{} area", title)))
            .alignment(Alignment::Center);
        Widget::render(text, inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_panel_state_updates() {
        let mut state = MultiPanelState::open(4.0);
        state.header.set_target(0.0);
        state.update(Duration::from_millis(250));
        assert!(state.header.current() < 1.0);
    }

    #[test]
    fn multi_panel_renders_without_panic() {
        let mut state = MultiPanelState::open(4.0);
        let widget = MultiPanel::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        StatefulWidget::render(widget, buf.area, &mut buf, &mut state);
    }
}
