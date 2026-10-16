//! A reusable, parameterized multi-panel widget.
//!
//! Renders a layout with header, footer, left sidebar, right sidebar, and main
//! content areas. Each panel's visibility is controlled by a [`Transition`]
//! stored in [`MultiPanelState`].
//!
//! Every panel can host custom content via a closure of type
//! `Fn(Rect, &mut Buffer)`. If no custom content is supplied, the panel renders
//! a default placeholder.

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiPanelState {
    pub header: Transition,
    pub left_sidebar: Transition,
    pub right_sidebar: Transition,
    pub footer: Transition,
}

impl MultiPanelState {
    /// Create a new state. Use `open = true` to start all panels open,
    /// `open = false` to start them closed.
    pub fn new(open: bool, speed: f64) -> Self {
        let value = if open { 1.0 } else { 0.0 };
        Self {
            header: Transition::new(value, speed),
            left_sidebar: Transition::new(value, speed),
            right_sidebar: Transition::new(value, speed),
            footer: Transition::new(value, speed),
        }
    }

    /// Create a new state with all panels open.
    pub fn open(speed: f64) -> Self {
        Self::new(true, speed)
    }

    /// Create a new state with all panels closed.
    pub fn closed(speed: f64) -> Self {
        Self::new(false, speed)
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

    /// Load state from the default config file, or return `None` if it does not
    /// exist or cannot be parsed.
    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let bytes = std::fs::read(&path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Save state to the default config file.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "could not determine config directory")
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_vec_pretty(self)?)
    }

    /// Path to the default state file: `~/.config/ratarelu/multi_panel_state.json`.
    pub fn config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("ratarelu").join("multi_panel_state.json"))
    }

    /// Current header/footer height in rows when `panel_height` is fully open.
    pub fn header_height(&self, panel_height: u16) -> u16 {
        (self.header.current() * panel_height as f64).round() as u16
    }

    /// Current footer height in rows when `panel_height` is fully open.
    pub fn footer_height(&self, panel_height: u16) -> u16 {
        (self.footer.current() * panel_height as f64).round() as u16
    }

    /// Current left sidebar width in columns for the given total width.
    pub fn left_sidebar_width(&self, total_width: u16, fraction: f64) -> u16 {
        (self.left_sidebar.current() * total_width as f64 * fraction).round() as u16
    }

    /// Current right sidebar width in columns for the given total width.
    pub fn right_sidebar_width(&self, total_width: u16, fraction: f64) -> u16 {
        (self.right_sidebar.current() * total_width as f64 * fraction).round() as u16
    }

    /// Current main content width in columns for the given total width.
    pub fn main_width(&self, total_width: u16, left_fraction: f64, right_fraction: f64) -> u16 {
        total_width
            .saturating_sub(self.left_sidebar_width(total_width, left_fraction))
            .saturating_sub(self.right_sidebar_width(total_width, right_fraction))
    }
}

/// Owned custom content renderer for a panel inside [`MultiPanel`].
///
/// Receives a mutable reference to the full [`MultiPanelState`] so the content
/// can react to other panels' open/close dimensions (or pass it along to
/// nested stateful widgets).
pub type PanelContent = Box<dyn Fn(&mut MultiPanelState, Rect, &mut Buffer)>;

/// Borrowed custom content renderer for a panel inside [`MultiPanel`].
pub type PanelContentRef<'a> = Option<&'a dyn Fn(&mut MultiPanelState, Rect, &mut Buffer)>;

/// Configuration for [`MultiPanel`].
#[derive(Default)]
pub struct MultiPanel {
    pub header_title: String,
    pub left_title: String,
    pub right_title: String,
    pub main_title: String,
    pub footer_title: String,
    pub header_content: Option<PanelContent>,
    pub left_content: Option<PanelContent>,
    pub right_content: Option<PanelContent>,
    pub main_content: Option<PanelContent>,
    pub footer_content: Option<PanelContent>,
    pub header_color: Color,
    pub left_color: Color,
    pub right_color: Color,
    pub footer_color: Color,
    pub main_color: Color,
    /// Height of the header when fully open, in rows.
    pub header_height: u16,
    /// Height of the footer when fully open, in rows.
    pub footer_height: u16,
    /// Width of the left sidebar as a fraction of total width (0.0–1.0).
    pub left_sidebar_width_fraction: f64,
    /// Width of the right sidebar as a fraction of total width (0.0–1.0).
    pub right_sidebar_width_fraction: f64,
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
            header_content: None,
            left_content: None,
            right_content: None,
            main_content: None,
            footer_content: None,
            header_color: Color::Blue,
            left_color: Color::Green,
            right_color: Color::Magenta,
            footer_color: Color::Yellow,
            main_color: Color::Reset,
            header_height: 5,
            footer_height: 5,
            left_sidebar_width_fraction: 0.25,
            right_sidebar_width_fraction: 0.25,
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

    /// Builder-style setter for custom header content.
    pub fn header_content<F>(mut self, content: F) -> Self
    where
        F: Fn(&mut MultiPanelState, Rect, &mut Buffer) + 'static,
    {
        self.header_content = Some(Box::new(content));
        self
    }

    /// Builder-style setter for custom left sidebar content.
    pub fn left_content<F>(mut self, content: F) -> Self
    where
        F: Fn(&mut MultiPanelState, Rect, &mut Buffer) + 'static,
    {
        self.left_content = Some(Box::new(content));
        self
    }

    /// Builder-style setter for custom right sidebar content.
    pub fn right_content<F>(mut self, content: F) -> Self
    where
        F: Fn(&mut MultiPanelState, Rect, &mut Buffer) + 'static,
    {
        self.right_content = Some(Box::new(content));
        self
    }

    /// Builder-style setter for custom main panel content.
    pub fn main_content<F>(mut self, content: F) -> Self
    where
        F: Fn(&mut MultiPanelState, Rect, &mut Buffer) + 'static,
    {
        self.main_content = Some(Box::new(content));
        self
    }

    /// Builder-style setter for custom footer content.
    pub fn footer_content<F>(mut self, content: F) -> Self
    where
        F: Fn(&mut MultiPanelState, Rect, &mut Buffer) + 'static,
    {
        self.footer_content = Some(Box::new(content));
        self
    }

    /// Builder-style setter for both header and footer height.
    pub fn panel_height(mut self, height: u16) -> Self {
        self.header_height = height;
        self.footer_height = height;
        self
    }

    /// Builder-style setter for header height.
    pub fn header_height(mut self, height: u16) -> Self {
        self.header_height = height;
        self
    }

    /// Builder-style setter for footer height.
    pub fn footer_height(mut self, height: u16) -> Self {
        self.footer_height = height;
        self
    }

    /// Builder-style setter for both sidebar width fractions.
    pub fn sidebar_width_fraction(mut self, fraction: f64) -> Self {
        let fraction = fraction.clamp(0.0, 1.0);
        self.left_sidebar_width_fraction = fraction;
        self.right_sidebar_width_fraction = fraction;
        self
    }

    /// Builder-style setter for left sidebar width fraction.
    pub fn left_sidebar_width_fraction(mut self, fraction: f64) -> Self {
        self.left_sidebar_width_fraction = fraction.clamp(0.0, 1.0);
        self
    }

    /// Builder-style setter for right sidebar width fraction.
    pub fn right_sidebar_width_fraction(mut self, fraction: f64) -> Self {
        self.right_sidebar_width_fraction = fraction.clamp(0.0, 1.0);
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

impl StatefulWidget for MultiPanel {
    type State = MultiPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_height = (state.header.current() * self.header_height as f64).round() as u16;
        let footer_height = (state.footer.current() * self.footer_height as f64).round() as u16;
        let left_width = (state.left_sidebar.current() * area.width as f64 * self.left_sidebar_width_fraction).round() as u16;
        let right_width = (state.right_sidebar.current() * area.width as f64 * self.right_sidebar_width_fraction).round() as u16;

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

        render_panel(
            buf,
            &self.header_title,
            vertical[0],
            self.header_color,
            state,
            self.header_content.as_deref(),
        );
        render_panel(
            buf,
            &self.left_title,
            horizontal[0],
            self.left_color,
            state,
            self.left_content.as_deref(),
        );
        render_panel(
            buf,
            &self.main_title,
            horizontal[1],
            self.main_color,
            state,
            self.main_content.as_deref(),
        );
        render_panel(
            buf,
            &self.right_title,
            horizontal[2],
            self.right_color,
            state,
            self.right_content.as_deref(),
        );
        render_panel(
            buf,
            &self.footer_title,
            vertical[2],
            self.footer_color,
            state,
            self.footer_content.as_deref(),
        );
    }
}

fn render_panel(
    buf: &mut Buffer,
    title: &str,
    area: Rect,
    color: Color,
    state: &mut MultiPanelState,
    content: PanelContentRef<'_>,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    Widget::render(block, area, buf);

    if inner.width > 0 && inner.height > 0 {
        match content {
            Some(render) => render(state, inner, buf),
            None => {
                let text = Paragraph::new(Text::from(format!("{} area", title)))
                    .alignment(Alignment::Center);
                Widget::render(text, inner, buf);
            }
        }
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

    #[test]
    fn multi_panel_renders_custom_content() {
        let mut state = MultiPanelState::open(4.0);
        let widget = MultiPanel::new().main_content(|_state, area, buf| {
            let p = Paragraph::new("custom").alignment(Alignment::Center);
            Widget::render(p, area, buf);
        });
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        StatefulWidget::render(widget, buf.area, &mut buf, &mut state);
    }

}
