//! A reusable, parameterized multi-panel widget.
//!
//! Renders a layout with header, footer, left sidebar, right sidebar, and main
//! content areas. Each panel's visibility is controlled by a [`Transition`]
//! stored in [`MultiPanelState`].
//!
//! Every panel can host custom content via a closure of type
//! `Fn(Rect, &mut Buffer)`. If no custom content is supplied, the panel renders
//! a default placeholder.

use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::animation::{ColorTransition, Transition};
use crate::file_tree::{FileTreeState, FileTreeWidget};
use crate::git::GitState;
use crate::keyboard::{KeyboardAction, KeyboardState, KeyboardWidget};
use crate::mouse::{ButtonState, is_mouse_over};/// Mutable state for [`MultiPanel`].
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

    /// Snap all panel transitions to their targets instantly.
    pub fn snap(&mut self) {
        self.header.snap();
        self.left_sidebar.snap();
        self.right_sidebar.snap();
        self.footer.snap();
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
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not determine config directory",
            )
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
    /// Minimum width of the left sidebar in columns.
    pub left_sidebar_min_width: u16,
    /// Minimum width of the right sidebar in columns.
    pub right_sidebar_min_width: u16,
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
            left_sidebar_min_width: 0,
            right_sidebar_min_width: 0,
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

    /// Builder-style setter for both sidebar minimum widths.
    pub fn sidebar_min_width(mut self, width: u16) -> Self {
        self.left_sidebar_min_width = width;
        self.right_sidebar_min_width = width;
        self
    }

    /// Builder-style setter for left sidebar minimum width.
    pub fn left_sidebar_min_width(mut self, width: u16) -> Self {
        self.left_sidebar_min_width = width;
        self
    }

    /// Builder-style setter for right sidebar minimum width.
    pub fn right_sidebar_min_width(mut self, width: u16) -> Self {
        self.right_sidebar_min_width = width;
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
        let left_width =
            ((state.left_sidebar.current() * area.width as f64 * self.left_sidebar_width_fraction)
                .round() as u16)
                .max(self.left_sidebar_min_width);
        let right_width = ((state.right_sidebar.current()
            * area.width as f64
            * self.right_sidebar_width_fraction)
            .round() as u16)
            .max(self.right_sidebar_min_width);

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
    let inner = if title.is_empty() {
        area
    } else {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color));
        let inner = block.inner(area);
        Widget::render(block, area, buf);
        inner
    };

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

// ============================================================
// Reusable multi-panel application driver
// ============================================================

/// Keys simulated during test mode.
pub const TEST_KEYS: [KeyCode; 5] = [
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Char(' '),
];

fn help_text() -> Text<'static> {
    let mut lines = Vec::new();

    let rows = [
        ("t", "toggle header"),
        ("l / a / q / p", "toggle left panel"),
        ("r / u", "toggle right panel"),
        ("f / h", "toggle footer"),
        ("Space", "toggle all panels"),
        ("↑ / ↓", "boost panels (left closed) / file nav (left open)"),
        ("← / →", "boost panels (left closed) / file nav (left open)"),
        ("Enter", "load selected file into main view"),
        ("Right", "activate main file view"),
        ("Esc", "de-boost / deactivate / close overlays"),
        ("n", "toggle line numbers"),
        ("e", "open selected file in $EDITOR"),
        ("\\", "cycle settings / help"),
        ("Shift+Q / Shift+P", "quit"),
        ("Shift+A", "hide / show inner grid"),
        ("Shift+T", "start / stop automated test matrix"),
        ("Shift+C", "toggle color transition demo"),
        ("Shift+M", "toggle chart dashboard"),
    ];

    lines.push(Line::default());
    lines.push(Line::default());

    for (key, action) in rows {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{:>12}", key), Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(action, Style::default().fg(Color::White)),
        ]));
    }

    Text::from(lines)
}

/// Build a human-readable summary of the app's current state for the help
/// overlay.
fn state_help_text(app: &MultiPanelApp) -> String {
    let bits = state_bits(app);
    let panel = |bit: u8| {
        if (bits >> bit) & 1 == 1 {
            "open"
        } else {
            "closed"
        }
    };
    let boost = |bit: u8| if (bits >> bit) & 1 == 1 { "on" } else { "off" };
    format!(
        "State bits: {:08b}\n\
        Top: {}  Top boost: {}  Left: {}  Left boost: {}\n\
        Right boost: {}  Right: {}  Bottom boost: {}  Bottom: {}",
        bits,
        panel(BIT_TOP),
        boost(BIT_TOP_BOOST),
        panel(BIT_LEFT),
        boost(BIT_LEFT_BOOST),
        boost(BIT_RIGHT_BOOST),
        panel(BIT_RIGHT),
        boost(BIT_BOTTOM_BOOST),
        panel(BIT_BOTTOM),
    )
}

/// A small widget that displays the current [`MultiPanelApp`] state.
#[derive(Debug, Clone)]
pub struct StateHelp {
    text: String,
}

impl StateHelp {
    /// Create a `StateHelp` widget from the current app state.
    pub fn new(app: &MultiPanelApp) -> Self {
        Self {
            text: state_help_text(app),
        }
    }
}

impl Widget for StateHelp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let p = Paragraph::new(self.text).alignment(Alignment::Left);
        Widget::render(p, area, buf);
    }
}

/// A cell in the interactive state help table.
#[derive(Debug, Clone, Copy)]
enum StateCell {
    Header,
    Left,
    Right,
    Footer,
    TopBoost,
    BottomBoost,
    LeftBoost,
    RightBoost,
    Grid,
}

impl StateCell {
    const ALL: [StateCell; 9] = [
        StateCell::Header,
        StateCell::TopBoost,
        StateCell::Left,
        StateCell::BottomBoost,
        StateCell::Right,
        StateCell::LeftBoost,
        StateCell::Footer,
        StateCell::RightBoost,
        StateCell::Grid,
    ];

    fn index(self) -> usize {
        match self {
            StateCell::Header => 0,
            StateCell::TopBoost => 1,
            StateCell::Left => 2,
            StateCell::BottomBoost => 3,
            StateCell::Right => 4,
            StateCell::LeftBoost => 5,
            StateCell::Footer => 6,
            StateCell::RightBoost => 7,
            StateCell::Grid => 8,
        }
    }

    fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    fn label(self, app: &MultiPanelApp) -> String {
        let (name, value) = match self {
            StateCell::Header => (
                "Header",
                if app.state.header.target() > 0.5 {
                    "open"
                } else {
                    "closed"
                },
            ),
            StateCell::Left => (
                "Left",
                if app.state.left_sidebar.target() > 0.5 {
                    "open"
                } else {
                    "closed"
                },
            ),
            StateCell::Right => (
                "Right",
                if app.state.right_sidebar.target() > 0.5 {
                    "open"
                } else {
                    "closed"
                },
            ),
            StateCell::Footer => (
                "Footer",
                if app.state.footer.target() > 0.5 {
                    "open"
                } else {
                    "closed"
                },
            ),
            StateCell::TopBoost => ("Top boost", if app.top_boost { "on" } else { "off" }),
            StateCell::BottomBoost => ("Bottom boost", if app.bottom_boost { "on" } else { "off" }),
            StateCell::LeftBoost => ("Left boost", if app.left_boost { "on" } else { "off" }),
            StateCell::RightBoost => ("Right boost", if app.right_boost { "on" } else { "off" }),
            StateCell::Grid => ("Grid", if app.show_grid { "on" } else { "off" }),
        };
        format!("{}: {}", name, value)
    }

    fn is_on(self, app: &MultiPanelApp) -> bool {
        match self {
            StateCell::Header => app.state.header.target() > 0.5,
            StateCell::Left => app.state.left_sidebar.target() > 0.5,
            StateCell::Right => app.state.right_sidebar.target() > 0.5,
            StateCell::Footer => app.state.footer.target() > 0.5,
            StateCell::TopBoost => app.top_boost,
            StateCell::BottomBoost => app.bottom_boost,
            StateCell::LeftBoost => app.left_boost,
            StateCell::RightBoost => app.right_boost,
            StateCell::Grid => app.show_grid,
        }
    }

    fn toggle(self, app: &mut MultiPanelApp) {
        match self {
            StateCell::Header => app.state.header.toggle_between(0.0, 1.0),
            StateCell::Left => app.state.left_sidebar.toggle_between(0.0, 1.0),
            StateCell::Right => app.state.right_sidebar.toggle_between(0.0, 1.0),
            StateCell::Footer => app.state.footer.toggle_between(0.0, 1.0),
            StateCell::TopBoost => app.top_boost = !app.top_boost,
            StateCell::BottomBoost => app.bottom_boost = !app.bottom_boost,
            StateCell::LeftBoost => app.left_boost = !app.left_boost,
            StateCell::RightBoost => app.right_boost = !app.right_boost,
            StateCell::Grid => app.show_grid = !app.show_grid,
        }
    }
}

/// State for [`StateHelpTable`], tracking per-cell button hit areas and visual
/// feedback.
#[derive(Debug, Default, Clone)]
pub struct StateHelpTableState {
    button_states: [ButtonState; 9],
}

impl StateHelpTableState {
    /// Handle a mouse event, returning `true` if it was consumed. Clicks on a
    /// table cell toggle the corresponding panel or boost flag in `app`.
    pub fn handle_mouse(&mut self, event: MouseEvent, app: &mut MultiPanelApp) -> bool {
        let mut consumed = false;
        let mut clicked: Option<StateCell> = None;
        for (i, btn) in self.button_states.iter_mut().enumerate() {
            if btn.handle_mouse(event) {
                consumed = true;
            }
            if matches!(event.kind, MouseEventKind::Down(MouseButton::Left))
                && let Some(area) = btn.last_area
                && is_mouse_over(area, event.column, event.row)
            {
                clicked = StateCell::from_index(i);
            }
        }
        if let Some(cell) = clicked {
            cell.toggle(app);
            // Reset visual states after a click; the next move event will
            // restore hover highlighting.
            for btn in &mut self.button_states {
                btn.state = crate::mouse::VisualState::Normal;
            }
            consumed = true;
        }
        consumed
    }
}

/// A 2×4 interactive table of panel and boost states.
#[derive(Debug, Clone)]
pub struct StateHelpTable {
    bits: u8,
    cells: [(StateCell, String, bool); 9],
}

impl StateHelpTable {
    /// Create a table from the current app state.
    pub fn new(app: &MultiPanelApp) -> Self {
        let cells = StateCell::ALL.map(|cell| (cell, cell.label(app), cell.is_on(app)));
        Self {
            bits: state_bits(app),
            cells,
        }
    }
}

impl StatefulWidget for StateHelpTable {
    type State = StateHelpTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(4)])
            .split(area);

        let bits_line =
            Paragraph::new(format!("State bits: {:08b}", self.bits)).alignment(Alignment::Center);
        Widget::render(bits_line, sections[0], buf);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
            ])
            .split(sections[1]);

        let mut render_cell = |i: usize, area: Rect| {
            let (cell, label, is_on) = &self.cells[i];
            state.button_states[cell.index()].last_area = Some(area);

            let border_color = if *is_on { Color::Green } else { Color::Magenta };
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color));
            let inner = block.inner(area);
            Widget::render(block, area, buf);
            if inner.width > 0 && inner.height > 0 {
                let p = Paragraph::new(Line::from(label.as_str())).alignment(Alignment::Center);
                Widget::render(p, inner, buf);
            }
        };

        render_cell(0, rows[0]); // Header
        render_cell(1, rows[1]); // Top boost

        // Middle row: left and right options combined, with Grid in the center.
        let mid_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(rows[2]);
        render_cell(2, mid_cols[0]); // Left
        render_cell(5, mid_cols[1]); // Left boost
        render_cell(8, mid_cols[2]); // Grid
        render_cell(7, mid_cols[3]); // Right boost
        render_cell(4, mid_cols[4]); // Right

        render_cell(3, rows[3]); // Bottom boost
        render_cell(6, rows[4]); // Footer
    }
}

// State bit indices for matrix operations.
//
// Bits are laid out MSB -> LSB as:
//   Top | Top boost | Left | Left boost | Right boost | Right | Bottom boost | Bottom
const BIT_TOP: u8 = 7;
const BIT_TOP_BOOST: u8 = 6;
const BIT_LEFT: u8 = 5;
const BIT_LEFT_BOOST: u8 = 4;
const BIT_RIGHT_BOOST: u8 = 3;
const BIT_RIGHT: u8 = 2;
const BIT_BOTTOM_BOOST: u8 = 1;
const BIT_BOTTOM: u8 = 0;

/// Configuration for [`MultiPanelApp`].
///
/// Lets callers supply their own [`MultiPanel`] widget builder and choose where
/// state is persisted.
pub struct MultiPanelAppConfig {
    /// Builder that produces the [`MultiPanel`] widget used for rendering.
    ///
    /// The builder receives the app so it can react to boost flags and test
    /// mode. A fresh widget is built each frame because Ratatui widgets are
    /// consumed when rendered.
    pub widget_builder: Box<dyn Fn(&MultiPanelApp) -> MultiPanel>,
    /// Optional path for `load`/`save`. If `None`, the default config directory
    /// is used.
    pub save_path: Option<std::path::PathBuf>,
}

impl Default for MultiPanelAppConfig {
    fn default() -> Self {
        Self::demo()
    }
}

impl MultiPanelAppConfig {
    /// Create a config with the built-in demo layout.
    pub fn demo() -> Self {
        Self {
            widget_builder: Box::new(demo_widget_builder),
            save_path: None,
        }
    }

    /// Create a config that renders the help overlay as a multi-panel layout.
    pub fn help() -> Self {
        Self {
            widget_builder: Box::new(help_widget_builder),
            save_path: None,
        }
    }
}

/// Full mutable application state for a multi-panel TUI, including boost flags
/// and an optional automated test mode.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MultiPanelApp {
    pub state: MultiPanelState,
    pub left_boost: bool,
    pub right_boost: bool,
    pub top_boost: bool,
    pub bottom_boost: bool,
    /// Whether the inner neighbor-report grid is visible.
    #[serde(default = "default_show_grid")]
    pub show_grid: bool,
    /// Shared state for the file tree rendered in the left panel.
    #[serde(skip)]
    pub file_tree_state: Rc<RefCell<FileTreeState>>,
    /// Persisted file tree state.
    #[serde(default)]
    pub file_tree_data: FileTreeState,
    /// Per-file view scroll positions.
    #[serde(default)]
    pub file_view_scrolls: HashMap<std::path::PathBuf, usize>,
    /// Async git state (if inside a git repo).
    #[serde(skip)]
    pub git: Option<GitState>,
    /// Currently selected file path (loaded on Enter).
    #[serde(default)]
    pub selected_file: Option<std::path::PathBuf>,
    /// Preview text of the selected file.
    #[serde(skip)]
    pub file_preview: String,
    /// Whether the main file view is active (receives scroll keys).
    #[serde(skip)]
    pub main_active: bool,
    /// Scroll offset for the main file view.
    #[serde(skip)]
    pub main_scroll: usize,
    /// Boost flag for the main file view (full-screen).
    #[serde(skip)]
    pub main_boost: bool,
    /// Show line numbers in the file view.
    #[serde(default = "default_line_numbers")]
    pub line_numbers: bool,
    /// Whether the full-screen help overlay is visible.
    #[serde(skip)]
    pub show_help: bool,
    /// Whether the full-screen settings overlay is visible.
    #[serde(skip)]
    pub show_settings: bool,
    /// Whether the color-transition demo is running.
    #[serde(skip)]
    pub color_demo: bool,
    #[serde(skip)]
    pub color_demo_transition: Option<ColorTransition>,
    #[serde(skip)]
    pub color_demo_timer: Duration,
    /// Whether the main area shows charts instead of the grid.
    #[serde(skip)]
    pub chart_mode: bool,
    #[serde(skip)]
    pub chart_time: f64,
    #[serde(skip)]
    pub test_mode: Option<TestMode>,
    #[serde(skip)]
    pub config: MultiPanelAppConfig,
    /// Interactive state help table state.
    #[serde(skip)]
    pub state_help_table_state: Rc<RefCell<StateHelpTableState>>,
    /// Keyboard widget state.
    #[serde(skip)]
    pub keyboard_state: Rc<RefCell<KeyboardState>>,
}

impl std::fmt::Debug for MultiPanelApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiPanelApp")
            .field("state", &self.state)
            .field("left_boost", &self.left_boost)
            .field("right_boost", &self.right_boost)
            .field("top_boost", &self.top_boost)
            .field("bottom_boost", &self.bottom_boost)
            .field("test_mode", &self.test_mode)
            .field("config", &"<MultiPanelAppConfig>")
            .field("show_settings", &self.show_settings)
            .field("file_tree_state", &"<FileTreeState>")
            .field("file_tree_data", &"<FileTreeState>")
            .field("keyboard_state", &"<KeyboardState>")
            .field("git", &self.git.is_some())
            .field("selected_file", &self.selected_file)
            .field("file_preview", &self.file_preview)
            .field("main_active", &self.main_active)
            .field("main_scroll", &self.main_scroll)
            .field("main_boost", &self.main_boost)
            .field("line_numbers", &self.line_numbers)
            .finish()
    }
}

/// Automated test mode state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestMode {
    pub state: u8,
    pub step: u8,
    pub timer: Duration,
}

impl Default for MultiPanelApp {
    fn default() -> Self {
        Self::new()
    }
}

const fn default_show_grid() -> bool {
    true
}

const fn default_line_numbers() -> bool {
    true
}

const COLOR_DEMO_PALETTE: [Color; 6] = [
    Color::Rgb(255, 0, 0),
    Color::Rgb(255, 127, 0),
    Color::Rgb(255, 255, 0),
    Color::Rgb(0, 255, 0),
    Color::Rgb(0, 127, 255),
    Color::Rgb(127, 0, 255),
];

impl MultiPanelApp {
    /// Create a new app with the demo config. Tries to load persisted state;
    /// falls back to all panels open.
    pub fn new() -> Self {
        Self::with_config(MultiPanelAppConfig::demo())
    }

    /// Create a new app with a custom config.
    pub fn with_config(config: MultiPanelAppConfig) -> Self {
        let path = config.save_path.clone().or_else(Self::default_config_path);
        if let Some(p) = path
            && let Some(mut app) = load_app_from_path(&p)
        {
            // Always rescan the working directory so the tree reflects new/removed files.
            let fresh = default_file_tree_paths();
            app.file_tree_state.borrow_mut().set_paths(fresh);
            // Restore persisted collapse/selection/scroll state onto the fresh tree.
            if !app.file_tree_data.paths().is_empty() {
                app.file_tree_state
                    .borrow_mut()
                    .apply_persisted_state(&app.file_tree_data);
            }
            if let Some(path) = app.selected_file.clone() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    app.file_preview = content;
                }
                app.main_scroll = app.file_view_scrolls.get(&path).copied().unwrap_or(0);
            }
            if app.git.is_none() {
                app.git = GitState::new();
            }
            app.config = config;
            return app;
        }
        Self::from_state_with_config(MultiPanelState::open(4.0), config)
    }

    /// Create an app from an existing panel state and the demo config.
    pub fn from_state(state: MultiPanelState) -> Self {
        Self::from_state_with_config(state, MultiPanelAppConfig::demo())
    }

    /// Create an app from an existing panel state and a custom config.
    pub fn from_state_with_config(state: MultiPanelState, config: MultiPanelAppConfig) -> Self {
        Self {
            state,
            left_boost: false,
            right_boost: false,
            top_boost: true,
            bottom_boost: true,
            show_grid: true,
            file_tree_state: Rc::new(RefCell::new(FileTreeState::from_paths(
                default_file_tree_paths(),
            ))),
            file_tree_data: FileTreeState::default(),
            file_view_scrolls: HashMap::new(),
            git: GitState::new(),
            selected_file: None,
            file_preview: String::new(),
            main_active: false,
            main_scroll: 0,
            main_boost: false,
            line_numbers: true,
            show_help: false,
            show_settings: false,
            color_demo: false,
            color_demo_transition: None,
            color_demo_timer: Duration::ZERO,
            chart_mode: false,
            chart_time: 0.0,
            test_mode: None,
            config,
            state_help_table_state: Rc::new(RefCell::new(StateHelpTableState::default())),
            keyboard_state: Rc::new(RefCell::new(KeyboardState::new())),
        }
    }

    /// Load app state from the configured path, or the default location.
    pub fn load(&self) -> Option<Self> {
        let path = self
            .config
            .save_path
            .clone()
            .or_else(Self::default_config_path)?;
        let mut app = load_app_from_path(&path)?;
        app.config.save_path = self.config.save_path.clone();
        Some(app)
    }

    /// Save app state to the configured path, or the default location.
    pub fn save(&mut self) -> std::io::Result<()> {
        self.file_tree_data = self.file_tree_state.borrow().snapshot();
        let path = self
            .config
            .save_path
            .clone()
            .or_else(Self::default_config_path)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "could not determine config directory",
                )
            })?;
        save_to_path(self, &path)
    }

    /// Default state file path: `~/.config/ratatui-relu/multi_panel_app.json`.
    pub fn default_config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("ratatui-relu").join("multi_panel_app.json"))
    }

    /// Advance animations, automated test mode, color demo, and charts.
    pub fn update(&mut self, dt: Duration) {
        self.state.update(dt);

        if let Some(git) = &mut self.git {
            git.poll();
            let items = git.status_items.clone();
            self.file_tree_state.borrow_mut().set_git_status(items);
        }
        self.file_tree_data = self.file_tree_state.borrow().snapshot();

        if self.chart_mode {
            self.chart_time += dt.as_secs_f64();
        }

        if self.color_demo {
            if self.color_demo_transition.is_none() {
                self.color_demo_transition =
                    Some(ColorTransition::new(COLOR_DEMO_PALETTE[0], 120.0));
                self.color_demo_transition
                    .as_mut()
                    .unwrap()
                    .set_target(COLOR_DEMO_PALETTE[1]);
                self.color_demo_timer = Duration::ZERO;
            }
            if let Some(transition) = &mut self.color_demo_transition {
                transition.update(dt);
                self.color_demo_timer += dt;
                if self.color_demo_timer >= Duration::from_secs(2) && transition.is_done() {
                    self.color_demo_timer = Duration::ZERO;
                    let current_target = transition.target();
                    let next_index = COLOR_DEMO_PALETTE
                        .iter()
                        .position(|&c| c == current_target)
                        .map(|i| (i + 1) % COLOR_DEMO_PALETTE.len())
                        .unwrap_or(1);
                    transition.set_target(COLOR_DEMO_PALETTE[next_index]);
                }
            }
        }

        let mut advance: Option<(u8, u8)> = None;
        if let Some(test) = &mut self.test_mode {
            test.timer += dt;
            if test.timer >= Duration::from_millis(500) {
                //debuging artifacts
                test.timer = Duration::ZERO;
                advance = Some((test.state, test.step));
                test.step += 1;
                if test.step >= TEST_KEYS.len() as u8 {
                    test.step = 0;
                    test.state = test.state.wrapping_add(1);
                }
            }
        }
        if let Some((state, step)) = advance {
            if step == 0 {
                set_state_bits(self, state);
                self.state.snap();
            }
            self.handle_key(TEST_KEYS[step as usize]);
        }
    }

    /// Start automated test mode and verify the transition matrix first.
    ///
    /// The test begins from the current TUI state rather than resetting to
    /// state zero. The inner grid is hidden so the layout transitions are
    /// clearly visible.
    pub fn start_test_mode(&mut self) {
        if let Some(err) = verify_transition_matrix() {
            eprintln!("Transition matrix mismatch: {}", err);
        } else {
            //eprintln!("Transition matrix verified: all 1280 state-key cases pass");
        }
        let start = state_bits(self);
        self.test_mode = Some(TestMode {
            state: start,
            step: 0,
            timer: Duration::ZERO,
        });
        set_state_bits(self, start);
        self.state.snap();
    }

    /// Stop automated test mode and restore the inner grid.
    pub fn stop_test_mode(&mut self) {
        self.test_mode = None;
        self.show_grid = true;
    }

    /// Handle a key press (real or simulated).
    pub fn handle_key(&mut self, code: KeyCode) {
        // When the left panel is open it is active.
        if self.state.left_sidebar.target() > 0.5
            && matches!(
                code,
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Enter
            )
        {
            if self.main_active {
                match code {
                    KeyCode::Up => {
                        self.main_scroll = self.main_scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        self.main_scroll = self.main_scroll.saturating_add(1);
                    }
                    KeyCode::Left => {
                        if self.main_boost {
                            self.main_boost = false;
                            // TODO handle scrolling differently self.main_scroll = self.main_scroll.saturating_sub(20);
                        } else {
                            // TODO handle scrolling differently self.main_scroll = self.main_scroll.saturating_sub(20);
                        }
                    }
                    KeyCode::Right => {
                        // Boost the main view.
                        self.main_boost = !self.main_boost;
                    }
                    _ => {}
                }
                if let Some(path) = &self.selected_file {
                    self.file_view_scrolls.insert(path.clone(), self.main_scroll);
                }
                return;
            }

            // Right arrow on a selected file loads it and activates the main view.
            if code == KeyCode::Right {
                let selected = self
                    .file_tree_state
                    .borrow()
                    .selected_path()
                    .map(|p| p.to_path_buf());
                if let Some(path) = selected.filter(|p| p.is_file()) {
                    self.selected_file = Some(path.clone());
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| format!("Error reading file: {}", e));
                    self.file_preview = if content.len() > 100_000 {
                        let mut t = content[..100_000].to_string();
                        t.push_str("\n... (truncated)");
                        t
                    } else {
                        content
                    };
                    self.main_active = true;
                    self.main_scroll = self
                        .selected_file
                        .as_ref()
                        .and_then(|p| self.file_view_scrolls.get(p).copied())
                        .unwrap_or(0);
                    if let (Some(git), Some(path)) = (&mut self.git, self.selected_file.clone()) {
                        git.fetch_diff(&path);
                    }
                    return;
                }
            }

            if code == KeyCode::Enter {
                if let Some(path) = self.file_tree_state.borrow().selected_path() {
                    let path = path.to_path_buf();
                    self.selected_file = Some(path.clone());
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| format!("Error reading file: {}", e));
                    self.file_preview = if content.len() > 100_000 {
                        let mut t = content[..100_000].to_string();
                        t.push_str("\n... (truncated)");
                        t
                    } else {
                        content
                    };
                    self.main_active = true;
                    self.main_scroll = self
                        .selected_file
                        .as_ref()
                        .and_then(|p| self.file_view_scrolls.get(p).copied())
                        .unwrap_or(0);
                    if let (Some(git), Some(path)) = (&mut self.git, self.selected_file.clone()) {
                        git.fetch_diff(&path);
                    }
                }
                return;
            }

            // Otherwise navigate the file tree.
            self.file_tree_state.borrow_mut().handle_key(code);
            return;
        }

        // Load the selected file into the main panel on Enter (left closed).
        if matches!(code, KeyCode::Enter) {
            if let Some(path) = self.file_tree_state.borrow().selected_path() {
                let path = path.to_path_buf();
                self.selected_file = Some(path.clone());
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| format!("Error reading file: {}", e));
                self.file_preview = if content.len() > 100_000 {
                    let mut t = content[..100_000].to_string();
                    t.push_str("\n... (truncated)");
                    t
                } else {
                    content
                };
                if let (Some(git), Some(path)) = (&mut self.git, self.selected_file.clone()) {
                    git.fetch_diff(&path);
                }
            }
            return;
        }

        // Data-driven panel toggle keys (symmetrical: 'l' and 'a' both toggle left).
        const PANEL_TOGGLE_KEYS: &[(char, usize)] = &[
            ('t', 0),
            ('l', 1),
            ('a', 1),
            ('q', 1),
            ('p', 1),
            ('r', 2),
            ('u', 2),
            ('f', 3),
            ('h', 3),
        ];
        if let KeyCode::Char(c) = code
            && let Some((_, panel)) = PANEL_TOGGLE_KEYS.iter().find(|(k, _)| *k == c)
        {
            match panel {
                    0 => self.state.header.toggle_between(0.0, 1.0),
                    1 => self.state.left_sidebar.toggle_between(0.0, 1.0),
                    2 => self.state.right_sidebar.toggle_between(0.0, 1.0),
                    3 => self.state.footer.toggle_between(0.0, 1.0),
                    _ => {}
                }
                return;
            }

        match code {
            // A = Automated Test Mode
            KeyCode::Char('A') => {
                if self.test_mode.is_some() {
                    self.stop_test_mode();
                } else {
                    self.start_test_mode();
                }
            }
            // G = Grid Mode
            KeyCode::Char('G') => self.show_grid = !self.show_grid,
            // Color demo
            KeyCode::Char('C') => self.color_demo = !self.color_demo,
            // Chart mode
            KeyCode::Char('M') => self.chart_mode = !self.chart_mode,
            KeyCode::Char(' ') => self.state.toggle_all(),
            KeyCode::Char('n') => self.line_numbers = !self.line_numbers,
            // File tree navigation in the left panel.
            KeyCode::Char('j') => {
                self.file_tree_state.borrow_mut().handle_key(KeyCode::Down);
            }
            KeyCode::Char('k') => {
                self.file_tree_state.borrow_mut().handle_key(KeyCode::Up);
            }
            KeyCode::Char('h') => {
                self.file_tree_state.borrow_mut().handle_key(KeyCode::Left);
            }
            KeyCode::Char('L') => {
                self.file_tree_state.borrow_mut().handle_key(KeyCode::Right);
            } // right handed                 // left handed
            KeyCode::Char('\\') => {
                if self.show_help {
                    self.show_help = false;
                } else if self.show_settings {
                    self.show_settings = false;
                    self.show_help = true;
                } else {
                    self.show_settings = true;
                }
            }
            KeyCode::Up => {
                let top_closed = self.state.header.target() < 0.5;
                let bottom_open = self.state.footer.target() > 0.5;
                if top_closed && bottom_open {
                    self.bottom_boost = !self.bottom_boost;
                } else if top_closed {
                    self.state.footer.set_target(1.0);
                } else {
                    self.state.header.toggle_between(0.0, 1.0);
                }
            }
            KeyCode::Down => {
                let bottom_closed = self.state.footer.target() < 0.5;
                let top_open = self.state.header.target() > 0.5;
                if bottom_closed && top_open {
                    self.top_boost = !self.top_boost;
                } else if bottom_closed {
                    self.state.header.set_target(1.0);
                } else {
                    self.state.footer.toggle_between(0.0, 1.0);
                }
            }
            KeyCode::Right => {
                let right_closed = self.state.right_sidebar.target() < 0.5;
                let left_open = self.state.left_sidebar.target() > 0.5;
                if right_closed && left_open {
                    self.left_boost = !self.left_boost;
                } else if right_closed {
                    self.state.left_sidebar.set_target(1.0);
                } else {
                    self.state.right_sidebar.toggle_between(0.0, 1.0);
                }
            }
            KeyCode::Left => {
                let left_closed = self.state.left_sidebar.target() < 0.5;
                let right_open = self.state.right_sidebar.target() > 0.5;
                if left_closed && right_open {
                    self.right_boost = !self.right_boost;
                } else if left_closed {
                    self.state.right_sidebar.set_target(1.0);
                } else {
                    self.state.left_sidebar.toggle_between(0.0, 1.0);
                }
            }
            _ => {}
        }
    }

    /// Handle a full key event, distinguishing `Shift+` combinations.
    ///
    /// Non-shifted keys are forwarded to [`Self::handle_key`]. Simulated keys
    /// (arrow keys used by test mode) should keep using [`Self::handle_key`].
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;
        match (key.code, key.modifiers) {
            // Grid Mode
            (KeyCode::Char('g'), m) if m.contains(KeyModifiers::SHIFT) => {
                self.show_grid = !self.show_grid;
            }
            // Test Mode
            (KeyCode::Char('t'), m) if m.contains(KeyModifiers::SHIFT) => {
                if self.test_mode.is_some() {
                    self.stop_test_mode();
                } else {
                    self.start_test_mode();
                }
            }
            // Color demo
            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::SHIFT) => {
                self.color_demo = !self.color_demo;
            }
            // Chart mode
            (KeyCode::Char('m'), m) if m.contains(KeyModifiers::SHIFT) => {
                self.chart_mode = !self.chart_mode;
            }
            _ => self.handle_key(key.code),
        }
    }

    /// Handle a mouse event. When the help overlay is open, clicks on the
    /// interactive state table toggle the corresponding panel or boost flag.
    pub fn handle_mouse(&mut self, event: MouseEvent) {
        if self.show_help {
            let state = self.state_help_table_state.clone();
            state.borrow_mut().handle_mouse(event, self);
            let kb = self.keyboard_state.clone();
            if let Some(action) = kb.borrow_mut().handle_mouse(event) {
                match action {
                    KeyboardAction::Char(c) => self.handle_key(KeyCode::Char(c)),
                    KeyboardAction::Space => self.handle_key(KeyCode::Char(' ')),
                }
            }
        }
    }

    /// Render the app into the given frame using the configured widget builder.
    pub fn render(&mut self, f: &mut Frame) {
        if self.show_settings {
            let area = f.area();
            Clear.render(area, f.buffer_mut());
            let mut settings_state = MultiPanelState::open(8.0);
            let widget = settings_widget_builder(self);
            f.render_stateful_widget(widget, area, &mut settings_state);
        } else if self.show_help {
            let area = f.area();
            Clear.render(area, f.buffer_mut());
            let mut help_state = MultiPanelState::open(8.0);
            let help_config = MultiPanelAppConfig::help();
            let widget = (help_config.widget_builder)(self);
            f.render_stateful_widget(widget, area, &mut help_state);
        } else {
            let widget = (self.config.widget_builder)(self);
            f.render_stateful_widget(widget, f.area(), &mut self.state);
        }
    }

    /// Current left sidebar width fraction, including boost.
    pub fn left_sidebar_fraction(&self) -> f64 {
        if self.left_boost { 0.50 } else { 0.25 }
    }

    /// Current right sidebar width fraction, including boost.
    pub fn right_sidebar_fraction(&self) -> f64 {
        if self.right_boost { 0.50 } else { 0.25 }
    }

    /// Current header height, including boost.
    pub fn header_height(&self) -> u16 {
        if self.top_boost { 10 } else { 5 }
    }

    /// Current footer height, including boost.
    pub fn footer_height(&self) -> u16 {
        if self.bottom_boost { 10 } else { 5 }
    }
}

fn bit_display(bits: u8) -> String {
    (0..8)
        .map(|i| {
            let bit = (bits >> (7 - i)) & 1;
            if bit == 1 { '1' } else { '0' }
        })
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn default_file_tree_paths() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &std::path::Path, depth: usize, out: &mut Vec<std::path::PathBuf>) {
        if depth == 0 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == "target" || name == ".git" || name == "node_modules" {
                continue;
            }
            if p.is_symlink() {
                // Do not follow symlinks to avoid loops; list them as leaves.
                out.push(p);
            } else if p.is_dir() {
                walk(&p, depth - 1, out);
            } else {
                out.push(p);
            }
        }
    }
    walk(std::path::Path::new("."), 3, &mut out);
    out
}

fn load_app_from_path(path: &std::path::Path) -> Option<MultiPanelApp> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn save_to_path(app: &MultiPanelApp, path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(app)?)
}

/// The built-in demo widget builder used by [`MultiPanelAppConfig::demo`].
pub fn demo_widget_builder(app: &MultiPanelApp) -> MultiPanel {
    let left_fraction = app.left_sidebar_fraction();
    let right_fraction = app.right_sidebar_fraction();
    let header_height = app.header_height();
    let footer_height = app.footer_height();
    let show_grid = app.show_grid;
    let chart_mode = app.chart_mode;
    let file_tree_state = app.file_tree_state.clone();
    let chart_time = app.chart_time;
    let file_preview = app.file_preview.clone();
    let selected_file = app.selected_file.clone();
    let main_scroll = app.main_scroll;
    let main_boost = app.main_boost;
    let line_numbers = app.line_numbers;
    let (left_fraction, right_fraction, header_height, footer_height) = if main_boost {
        (0.0, 0.0, 0u16, 0u16)
    } else {
        (left_fraction, right_fraction, header_height, footer_height)
    };
    let git_diff_text = app
        .git
        .as_ref()
        .and_then(|g| g.diff.as_ref())
        .filter(|(params, _)| {
            selected_file
                .as_ref()
                .map(|p| params.path == p.to_string_lossy())
                .unwrap_or(false)
        })
        .map(|(_, diff)| {
            let mut out = String::new();
            for hunk in &diff.hunks {
                for line in &hunk.lines {
                    out.push_str(&line.content);
                    out.push('\n');
                }
            }
            out
        })
        .filter(|s| !s.trim().is_empty());
    let color_demo_color = app.color_demo_transition.as_ref().map(|ct| ct.current());
    let test_bits = app.test_mode.as_ref().map(|_| state_bits(app));
    let test_label = app.test_mode.as_ref().map(|t| {
        format!(
            "TEST MODE state {}/256 key {}/{}",
            t.state.wrapping_add(1),
            t.step + 1,
            TEST_KEYS.len(),
        )
    });
    let help = "[t/l/r/f] toggle  [Space] all  [↑↓←→] boost  [Shift+A] automate tui  [Shift+T] test  [Shift+C] colors  [Shift+M] charts  [\\] help  [q] quit";

    MultiPanel::new()
        .header_title("Header")
        .left_title("Left")
        .right_title("Right")
        .main_title("")
        .footer_title("Footer")
        .header_content(move |_state, area, buf| {
            let top = Rect::new(area.x, area.y, area.width, 1.min(area.height));
            let p = Paragraph::new("↑ top").alignment(Alignment::Center);
            Widget::render(p, top, buf);
        })
        .left_content({
            let file_tree_state = file_tree_state.clone();
            move |_state, area, buf| {
                use ratatui::widgets::StatefulWidget;
                let mut state = file_tree_state.borrow_mut();
                let widget = FileTreeWidget::new();
                StatefulWidget::render(widget, area, buf, &mut state);
            }
        })
        .right_content(|_state, area, buf| {
            let p = Paragraph::new("right →").alignment(Alignment::Right);
            Widget::render(p, area, buf);
        })
        .main_content(move |state, area, buf| {
            use crate::chart::ChartDemo;
            use crate::report_grid::NeighborReportGrid;
            use ratatui::widgets::Clear;
            if let Some(diff_text) = &git_diff_text {
                let title = selected_file
                    .as_ref()
                    .map(|p| {
                        p.canonicalize()
                            .unwrap_or_else(|_| p.clone())
                            .display()
                            .to_string()
                    })
                    .unwrap_or_else(|| "diff".to_string());
                let display = if line_numbers {
                    diff_text
                        .lines()
                        .enumerate()
                        .map(|(i, l)| format!("{:>4}  {}", i + 1, l))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    diff_text.clone()
                };
                let block = ratatui::widgets::Block::bordered().title(title);
                let inner = block.inner(area);
                ratatui::widgets::Widget::render(block, area, buf);
                if inner.width > 0 && inner.height > 0 {
                    let p = ratatui::widgets::Paragraph::new(display)
                        .wrap(ratatui::widgets::Wrap { trim: false })
                        .scroll((main_scroll as u16, 0));
                    ratatui::widgets::Widget::render(p, inner, buf);
                }
            } else if !file_preview.is_empty() {
                let title = selected_file
                    .as_ref()
                    .map(|p| {
                        p.canonicalize()
                            .unwrap_or_else(|_| p.clone())
                            .display()
                            .to_string()
                    })
                    .unwrap_or_else(|| "preview".to_string());
                let display = if line_numbers {
                    file_preview
                        .lines()
                        .enumerate()
                        .map(|(i, l)| format!("{:>4}  {}", i + 1, l))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    file_preview.clone()
                };
                let block = ratatui::widgets::Block::bordered().title(title);
                let inner = block.inner(area);
                ratatui::widgets::Widget::render(block, area, buf);
                if inner.width > 0 && inner.height > 0 {
                    let p = ratatui::widgets::Paragraph::new(display)
                        .wrap(ratatui::widgets::Wrap { trim: false })
                        .scroll((main_scroll as u16, 0));
                    ratatui::widgets::Widget::render(p, inner, buf);
                }
            } else if chart_mode {
                Widget::render(ChartDemo::new(chart_time), area, buf);
            } else if show_grid {
                let grid = NeighborReportGrid::new(
                    left_fraction,
                    right_fraction,
                    header_height,
                    footer_height,
                )
                .center_content({
                    if let Some(label) = test_label.clone() {
                        let bits = test_bits.unwrap_or(0);
                        format!("{}\n{}", label, bit_display(bits))
                    } else {
                        format!("{}x{}", area.width, area.height)
                    }
                })
                .cell_color(color_demo_color.unwrap_or(Color::Reset));
                StatefulWidget::render(grid, area, buf, state);
            } else {
                Clear.render(area, buf);
            }
        })
        .footer_content(move |_state, area, buf| {
            let bottom = Rect::new(
                area.x,
                area.y + area.height.saturating_sub(1),
                area.width,
                1.min(area.height),
            );
            let p = Paragraph::new(help).alignment(Alignment::Center);
            Widget::render(p, bottom, buf);
        })
        .header_color(color_demo_color.unwrap_or(Color::Blue))
        .left_color(color_demo_color.unwrap_or(Color::Green))
        .right_color(color_demo_color.unwrap_or(Color::Magenta))
        .footer_color(color_demo_color.unwrap_or(Color::Yellow))
        .main_color(color_demo_color.unwrap_or(Color::Reset))
        .header_height(header_height)
        .footer_height(footer_height)
        .left_sidebar_width_fraction(left_fraction)
        .right_sidebar_width_fraction(right_fraction)
}

/// The built-in settings widget builder. Renders settings inside a
/// multi-panel layout matching the help overlay.
pub fn settings_widget_builder(_app: &MultiPanelApp) -> MultiPanel {
    let left_text = "\n\n    l/a/q/p: left panel\n    r/u: right panel\n    f/h: footer\n    t: header\n    Space: all panels\n    n: line numbers\n    e: open in editor";
    let right_text = "\n\n    Press \\ again for help\n    Esc/q to close";
    let line_width = |text: &str| {
        text.lines()
            .map(|line| line.len() as u16)
            .max()
            .unwrap_or(0)
            + 5
    };

    MultiPanel::new()
        .header_title("Settings")
        .left_title("Options")
        .right_title("Info")
        .main_title("")
        .footer_title("Press \\ or q/Esc to dismiss")
        .header_content(|_state, area, buf| {
            let p = Paragraph::new("ratatui-relu settings")
                .alignment(Alignment::Center)
                .fg(Color::Cyan);
            Widget::render(p, area, buf);
        })
        .left_content(move |_state, area, buf| {
            let p = Paragraph::new(left_text).alignment(Alignment::Left);
            Widget::render(p, area, buf);
        })
        .right_content(move |_state, area, buf| {
            let p = Paragraph::new(right_text).alignment(Alignment::Left);
            Widget::render(p, area, buf);
        })
        .main_content({
            let table = StateHelpTable::new(_app);
            let table_state = _app.state_help_table_state.clone();
            move |_state, area, buf| {
                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                    .split(area);

                let shortcuts_block = Block::bordered()
                    .title("Shortcuts")
                    .title_alignment(Alignment::Left);
                let shortcuts_inner = shortcuts_block.inner(rows[0]);
                Widget::render(shortcuts_block, rows[0], buf);
                Widget::render(
                    Paragraph::new(help_text()).alignment(Alignment::Left),
                    shortcuts_inner,
                    buf,
                );

                let toggles_block = Block::bordered()
                    .title("Toggles")
                    .title_alignment(Alignment::Left);
                let toggles_inner = toggles_block.inner(rows[1]);
                Widget::render(toggles_block, rows[1], buf);
                StatefulWidget::render(
                    table.clone(),
                    toggles_inner,
                    buf,
                    &mut table_state.borrow_mut(),
                );
            }
        })
        .footer_content(|_state, area, buf| {
            let p = Paragraph::new("Press \\ or q/Esc to dismiss").alignment(Alignment::Center);
            Widget::render(p, area, buf);
        })
        .header_color(Color::Cyan)
        .left_color(Color::Green)
        .right_color(Color::Magenta)
        .main_color(Color::Reset)
        .footer_color(Color::Yellow)
        .panel_height(3)
        .sidebar_width_fraction(0.25)
        .left_sidebar_min_width(line_width(left_text))
        .right_sidebar_min_width(line_width(right_text))
}

/// The built-in help widget builder. Renders the help text inside a
/// multi-panel layout.
pub fn help_widget_builder(_app: &MultiPanelApp) -> MultiPanel {
    let left_text = "\n\n    ReLU-gated layout transitions\n    Smooth animations\n    Multi-panel widget\n    FFI bindings";
    let right_text = "\n\n    Try Shift+T for test mode\n    Try Shift+C for colors\n    Try Shift+M for charts\n    Try Shift+G to hide grid\n   Try Shift+A to automate tui";
    let line_width = |text: &str| {
        text.lines()
            .map(|line| line.len() as u16)
            .max()
            .unwrap_or(0)
            + 5
    };

    MultiPanel::new()
        .header_title("Help")
        .left_title("About")
        .right_title("Tips")
        .main_title("Main")
        .footer_title("Press \\ to close")
        .header_content(|_state, area, buf| {
            let p = Paragraph::new("ratatui-relu help overlay")
                .alignment(Alignment::Center)
                .fg(Color::Cyan);
            Widget::render(p, area, buf);
        })
        .left_content(move |_state, area, buf| {
            let p = Paragraph::new(left_text).alignment(Alignment::Left);
            Widget::render(p, area, buf);
        })
        .right_content(move |_state, area, buf| {
            let p = Paragraph::new(right_text).alignment(Alignment::Left);
            Widget::render(p, area, buf);
        })
        .main_content({
            let keyboard_state = _app.keyboard_state.clone();
            move |_state, area, buf| {
                use ratatui::widgets::StatefulWidget;
                let widget = KeyboardWidget::default();
                let mut st = keyboard_state.borrow_mut();
                StatefulWidget::render(widget, area, buf, &mut st);
            }
        })
        .footer_content(|_state, area, buf| {
            let p = Paragraph::new("Press \\ or q/Esc to dismiss").alignment(Alignment::Center);
            Widget::render(p, area, buf);
        })
        .header_color(Color::Cyan)
        .left_color(Color::Green)
        .right_color(Color::Magenta)
        .main_color(Color::Reset)
        .footer_color(Color::Yellow)
        .panel_height(3)
        .sidebar_width_fraction(0.2)
        .left_sidebar_min_width(line_width(left_text))
        .right_sidebar_min_width(line_width(right_text))
}

/// Encode panel targets and boost flags into an 8-bit state vector.
pub fn state_bits(app: &MultiPanelApp) -> u8 {
    let panel_bit = |panel: &Transition, idx: u8| {
        if panel.target() > 0.5 { 1 << idx } else { 0 }
    };
    panel_bit(&app.state.header, BIT_TOP)
        | panel_bit(&app.state.left_sidebar, BIT_LEFT)
        | panel_bit(&app.state.right_sidebar, BIT_RIGHT)
        | panel_bit(&app.state.footer, BIT_BOTTOM)
        | if app.top_boost { 1 << BIT_TOP_BOOST } else { 0 }
        | if app.bottom_boost {
            1 << BIT_BOTTOM_BOOST
        } else {
            0
        }
        | if app.left_boost {
            1 << BIT_LEFT_BOOST
        } else {
            0
        }
        | if app.right_boost {
            1 << BIT_RIGHT_BOOST
        } else {
            0
        }
}

/// Apply an 8-bit state vector to the app (panels + boosts).
pub fn set_state_bits(app: &mut MultiPanelApp, bits: u8) {
    let open = |idx: u8| if (bits >> idx) & 1 == 1 { 1.0 } else { 0.0 };
    app.state.header.set_target(open(BIT_TOP));
    app.state.left_sidebar.set_target(open(BIT_LEFT));
    app.state.right_sidebar.set_target(open(BIT_RIGHT));
    app.state.footer.set_target(open(BIT_BOTTOM));
    app.top_boost = (bits >> BIT_TOP_BOOST) & 1 == 1;
    app.bottom_boost = (bits >> BIT_BOTTOM_BOOST) & 1 == 1;
    app.left_boost = (bits >> BIT_LEFT_BOOST) & 1 == 1;
    app.right_boost = (bits >> BIT_RIGHT_BOOST) & 1 == 1;
}

/// Transition matrix for arrow keys and Space over the full state space.
///
/// State is an 8-bit vector laid out MSB -> LSB as:
///   Top | Top boost | Left | Left boost | Right boost | Right | Bottom boost | Bottom
pub fn transition_matrix(state: u8, key: crossterm::event::KeyCode) -> u8 {
    let h = (state >> BIT_TOP) & 1;
    let l = (state >> BIT_LEFT) & 1;
    let r = (state >> BIT_RIGHT) & 1;
    let f = (state >> BIT_BOTTOM) & 1;
    let top_b = (state >> BIT_TOP_BOOST) & 1;
    let bot_b = (state >> BIT_BOTTOM_BOOST) & 1;
    let left_b = (state >> BIT_LEFT_BOOST) & 1;
    let right_b = (state >> BIT_RIGHT_BOOST) & 1;

    // When the left panel is open, arrow keys drive the file tree and do not
    // change panel/boost state.
    if l == 1
        && matches!(
            key,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        )
    {
        return state;
    }

    let (nh, nl, nr, nf, ntop_b, nbot_b, nleft_b, nright_b) = match key {
        KeyCode::Up => {
            if h == 0 && f == 1 {
                (h, l, r, f, top_b, bot_b ^ 1, left_b, right_b)
            } else if h == 0 {
                (h, l, r, 1, top_b, bot_b, left_b, right_b)
            } else {
                (0, l, r, f, top_b, bot_b, left_b, right_b)
            }
        }
        KeyCode::Down => {
            if f == 0 && h == 1 {
                (h, l, r, f, top_b ^ 1, bot_b, left_b, right_b)
            } else if f == 0 {
                (1, l, r, f, top_b, bot_b, left_b, right_b)
            } else {
                (h, l, r, 0, top_b, bot_b, left_b, right_b)
            }
        }
        KeyCode::Right => {
            if r == 0 && l == 1 {
                (h, l, r, f, top_b, bot_b, left_b ^ 1, right_b)
            } else if r == 0 {
                (h, 1, r, f, top_b, bot_b, left_b, right_b)
            } else {
                (h, l, 0, f, top_b, bot_b, left_b, right_b)
            }
        }
        KeyCode::Left => {
            if l == 0 && r == 1 {
                (h, l, r, f, top_b, bot_b, left_b, right_b ^ 1)
            } else if l == 0 {
                (h, l, 1, f, top_b, bot_b, left_b, right_b)
            } else {
                (h, 0, r, f, top_b, bot_b, left_b, right_b)
            }
        }
        KeyCode::Char(' ') => (h ^ 1, l ^ 1, r ^ 1, f ^ 1, top_b, bot_b, left_b, right_b),
        _ => (h, l, r, f, top_b, bot_b, left_b, right_b),
    };

    (nh << BIT_TOP)
        | (nl << BIT_LEFT)
        | (nr << BIT_RIGHT)
        | (nf << BIT_BOTTOM)
        | (ntop_b << BIT_TOP_BOOST)
        | (nbot_b << BIT_BOTTOM_BOOST)
        | (nleft_b << BIT_LEFT_BOOST)
        | (nright_b << BIT_RIGHT_BOOST)
}

/// Run the full 256×5 transition matrix and return the first mismatch, if any.
pub fn verify_transition_matrix() -> Option<String> {
    for state in 0..=255u8 {
        for key in TEST_KEYS {
            let mut app = MultiPanelApp::from_state(MultiPanelState::closed(100.0));
            set_state_bits(&mut app, state);
            app.handle_key(key);
            let expected = transition_matrix(state, key);
            let actual = state_bits(&app);
            if actual != expected {
                return Some(format!(
                    "state={:08b} key={:?} expected={:08b} actual={:08b}",
                    state, key, expected, actual
                ));
            }
        }
    }
    None
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

    #[test]
    fn transition_matrix_is_consistent() {
        assert!(verify_transition_matrix().is_none());
    }
}
