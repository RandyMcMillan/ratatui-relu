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

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::animation::{ColorTransition, Transition};

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
        let left_width =
            (state.left_sidebar.current() * area.width as f64 * self.left_sidebar_width_fraction)
                .round() as u16;
        let right_width =
            (state.right_sidebar.current() * area.width as f64 * self.right_sidebar_width_fraction)
                .round() as u16;

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

// ============================================================
// Reusable multi-panel application driver
// ============================================================

/// Keys simulated during test mode.
pub const TEST_KEYS: [KeyCode; 4] = [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down];

const HELP_TEXT: &str = "ratatui-relu shortcuts\n\n\
    h/l/r/f     toggle header / left / right / footer\n\
    Space       toggle all panels\n\
    ↑ / ↓       close top / bottom and boost opposite\n\
    ← / →       close side and boost opposite\n\
    Shift+H     hide / show inner grid\n\
    Shift+T     start / stop automated test matrix\n\
    Shift+C     toggle color transition demo\n\
    \\           toggle this help overlay\n\
    q / Esc     quit";

// State bit indices for matrix operations.
// Panel bits:
const BIT_HEADER: u8 = 0;
const BIT_LEFT: u8 = 1;
const BIT_RIGHT: u8 = 2;
const BIT_FOOTER: u8 = 3;
// Boost bits:
const BIT_TOP_BOOST: u8 = 4;
const BIT_BOTTOM_BOOST: u8 = 5;
const BIT_LEFT_BOOST: u8 = 6;
const BIT_RIGHT_BOOST: u8 = 7;

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
    #[serde(skip)]
    pub show_grid: bool,
    /// Whether the full-screen help overlay is visible.
    #[serde(skip)]
    pub show_help: bool,
    /// Whether the color-transition demo is running.
    #[serde(skip)]
    pub color_demo: bool,
    #[serde(skip)]
    pub color_demo_transition: Option<ColorTransition>,
    #[serde(skip)]
    pub color_demo_timer: Duration,
    #[serde(skip)]
    pub test_mode: Option<TestMode>,
    #[serde(skip)]
    pub config: MultiPanelAppConfig,
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
            left_boost: true,
            right_boost: true,
            top_boost: true,
            bottom_boost: true,
            show_grid: true,
            show_help: false,
            color_demo: false,
            color_demo_transition: None,
            color_demo_timer: Duration::ZERO,
            test_mode: None,
            config,
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
    pub fn save(&self) -> std::io::Result<()> {
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

    /// Advance animations, automated test mode, and color demo.
    pub fn update(&mut self, dt: Duration) {
        self.state.update(dt);

        if self.color_demo {
            if self.color_demo_transition.is_none() {
                self.color_demo_transition = Some(ColorTransition::new(
                    COLOR_DEMO_PALETTE[0],
                    120.0,
                ));
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
            //eprintln!("Transition matrix verified: all 1024 state-key cases pass");
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
        match code {
            // T = Test Mode
            KeyCode::Char('T') => {
                if self.test_mode.is_some() {
                    self.stop_test_mode();
                } else {
                    self.start_test_mode();
                }
            }
            // G = Grid Mode
            KeyCode::Char('G') => self.show_grid = !self.show_grid,
            // t = top
            KeyCode::Char('t') => self.state.header.toggle_between(0.0, 1.0),
            KeyCode::Char('l') => self.state.left_sidebar.toggle_between(0.0, 1.0),
            KeyCode::Char('r') => self.state.right_sidebar.toggle_between(0.0, 1.0),
            KeyCode::Char('f') => self.state.footer.toggle_between(0.0, 1.0),
            KeyCode::Char(' ') => self.state.toggle_all(),
            // right handed                 // left handed
            KeyCode::Char('\\') | KeyCode::Char('q') => self.show_help = !self.show_help,
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
            _ => self.handle_key(key.code),
        }
    }

    /// Render the app into the given frame using the configured widget builder.
    pub fn render(&mut self, f: &mut Frame) {
        if self.show_help {
            let area = f.area();
            Clear.render(area, f.buffer_mut());
            let help = Paragraph::new(HELP_TEXT)
                .alignment(Alignment::Center)
                .block(Block::default().title("Help").borders(Borders::ALL));
            f.render_widget(help, area);
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
    let color_demo_color = app.color_demo_transition.as_ref().map(|ct| ct.current());
    let test_label = app.test_mode.as_ref().map(|t| {
        let bits = state_bits(app);
        format!(
            "TEST MODE state {}/256 key {}/{} bits {:08b}",
            t.state.wrapping_add(1),
            t.step + 1,
            TEST_KEYS.len(),
            bits
        )
    });
    let help = "[h/l/r/f] toggle  [Space] all  [↑↓←→] boost  [Shift+H] hide grid  [Shift+T] test  [Shift+C] colors  [\\] help  [q] quit";

    MultiPanel::new()
        .header_title("Header")
        .left_title("Left")
        .right_title("Right")
        .main_title("Main")
        .footer_title("Footer")
        .header_content(move |_state, area, buf| {
            let top = Rect::new(area.x, area.y, area.width, 1.min(area.height));
            let p = Paragraph::new("↑ top").alignment(Alignment::Center);
            Widget::render(p, top, buf);
        })
        .left_content(|_state, area, buf| {
            let p = Paragraph::new("← left").alignment(Alignment::Left);
            Widget::render(p, area, buf);
        })
        .right_content(|_state, area, buf| {
            let p = Paragraph::new("right →").alignment(Alignment::Right);
            Widget::render(p, area, buf);
        })
        .main_content(move |state, area, buf| {
            use crate::report_grid::NeighborReportGrid;
            use ratatui::widgets::Clear;
            if show_grid {
                let grid = NeighborReportGrid::new(
                    left_fraction,
                    right_fraction,
                    header_height,
                    footer_height,
                )
                .center_content(
                    test_label
                        .clone()
                        .unwrap_or_else(|| format!("{}x{}", area.width, area.height)),
                )
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

/// Encode panel targets and boost flags into an 8-bit state vector.
pub fn state_bits(app: &MultiPanelApp) -> u8 {
    let panel_bit = |panel: &Transition, idx: u8| {
        if panel.target() > 0.5 { 1 << idx } else { 0 }
    };
    panel_bit(&app.state.header, BIT_HEADER)
        | panel_bit(&app.state.left_sidebar, BIT_LEFT)
        | panel_bit(&app.state.right_sidebar, BIT_RIGHT)
        | panel_bit(&app.state.footer, BIT_FOOTER)
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
    app.state.header.set_target(open(BIT_HEADER));
    app.state.left_sidebar.set_target(open(BIT_LEFT));
    app.state.right_sidebar.set_target(open(BIT_RIGHT));
    app.state.footer.set_target(open(BIT_FOOTER));
    app.top_boost = (bits >> BIT_TOP_BOOST) & 1 == 1;
    app.bottom_boost = (bits >> BIT_BOTTOM_BOOST) & 1 == 1;
    app.left_boost = (bits >> BIT_LEFT_BOOST) & 1 == 1;
    app.right_boost = (bits >> BIT_RIGHT_BOOST) & 1 == 1;
}

/// Transition matrix for arrow keys over the full state space.
///
/// State is an 8-bit vector: lower 4 bits are panel open/closed, upper 4 bits
/// are boost flags (top, bottom, left, right).
pub fn transition_matrix(state: u8, key: crossterm::event::KeyCode) -> u8 {
    let h = (state >> BIT_HEADER) & 1;
    let l = (state >> BIT_LEFT) & 1;
    let r = (state >> BIT_RIGHT) & 1;
    let f = (state >> BIT_FOOTER) & 1;
    let top_b = (state >> BIT_TOP_BOOST) & 1;
    let bot_b = (state >> BIT_BOTTOM_BOOST) & 1;
    let left_b = (state >> BIT_LEFT_BOOST) & 1;
    let right_b = (state >> BIT_RIGHT_BOOST) & 1;

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
        _ => (h, l, r, f, top_b, bot_b, left_b, right_b),
    };

    (nh << BIT_HEADER)
        | (nl << BIT_LEFT)
        | (nr << BIT_RIGHT)
        | (nf << BIT_FOOTER)
        | (ntop_b << BIT_TOP_BOOST)
        | (nbot_b << BIT_BOTTOM_BOOST)
        | (nleft_b << BIT_LEFT_BOOST)
        | (nright_b << BIT_RIGHT_BOOST)
}

/// Run the full 256×4 transition matrix and return the first mismatch, if any.
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
