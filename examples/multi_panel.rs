//! Multi-panel layout controlled by independent animated transitions.
//!
//! Demonstrates the reusable [`MultiPanel`] widget from `ratarelu::multi_panel`.
//!
//! Run in a real terminal with: cargo run --example multi_panel

use std::io::{self, stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::Color,
    Terminal,
};

use ratarelu::multi_panel::{MultiPanel, MultiPanelState};
use ratarelu::report_grid::NeighborReportGrid;

struct App {
    state: MultiPanelState,
    left_boost: bool,
    right_boost: bool,
    top_boost: bool,
    bottom_boost: bool,
    test_mode: Option<TestMode>,
    should_quit: bool,
}

struct TestMode {
    state: u8,
    step: u8,
    timer: Duration,
}

/// Keys simulated during each test permutation.
const TEST_KEYS: [KeyCode; 4] = [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down];

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

/// Encode panel targets and boost flags into an 8-bit state vector.
fn state_bits(app: &App) -> u8 {
    let panel_bit = |panel: &ratarelu::animation::Transition, idx: u8| {
        if panel.target() > 0.5 { 1 << idx } else { 0 }
    };
    panel_bit(&app.state.header, BIT_HEADER)
        | panel_bit(&app.state.left_sidebar, BIT_LEFT)
        | panel_bit(&app.state.right_sidebar, BIT_RIGHT)
        | panel_bit(&app.state.footer, BIT_FOOTER)
        | if app.top_boost { 1 << BIT_TOP_BOOST } else { 0 }
        | if app.bottom_boost { 1 << BIT_BOTTOM_BOOST } else { 0 }
        | if app.left_boost { 1 << BIT_LEFT_BOOST } else { 0 }
        | if app.right_boost { 1 << BIT_RIGHT_BOOST } else { 0 }
}

/// Apply an 8-bit state vector to the app (panels + boosts).
fn set_state_bits(app: &mut App, bits: u8) {
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
fn transition_matrix(state: u8, key: KeyCode) -> u8 {
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
                (h, l, r, f, top_b, bot_b ^ 1, left_b, right_b) // toggle bottom boost
            } else if h == 0 {
                (h, l, r, 1, top_b, bot_b, left_b, right_b) // open footer
            } else {
                (0, l, r, f, top_b, bot_b, left_b, right_b) // close header
            }
        }
        KeyCode::Down => {
            if f == 0 && h == 1 {
                (h, l, r, f, top_b ^ 1, bot_b, left_b, right_b) // toggle top boost
            } else if f == 0 {
                (1, l, r, f, top_b, bot_b, left_b, right_b) // open header
            } else {
                (h, l, r, 0, top_b, bot_b, left_b, right_b) // close footer
            }
        }
        KeyCode::Right => {
            if r == 0 && l == 1 {
                (h, l, r, f, top_b, bot_b, left_b ^ 1, right_b) // toggle left boost
            } else if r == 0 {
                (h, 1, r, f, top_b, bot_b, left_b, right_b) // open left
            } else {
                (h, l, 0, f, top_b, bot_b, left_b, right_b) // close right
            }
        }
        KeyCode::Left => {
            if l == 0 && r == 1 {
                (h, l, r, f, top_b, bot_b, left_b, right_b ^ 1) // toggle right boost
            } else if l == 0 {
                (h, l, 1, f, top_b, bot_b, left_b, right_b) // open right
            } else {
                (h, 0, r, f, top_b, bot_b, left_b, right_b) // close left
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
fn verify_transition_matrix() -> Option<String> {
    for state in 0..=255u8 {
        for key in TEST_KEYS {
            let mut app = App::from_state(MultiPanelState::closed(100.0));
            set_state_bits(&mut app, state);
            app.simulate_key(key);
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

impl App {
    fn new() -> Self {
        // Try to restore previous panel state; fall back to all panels open.
        let state = MultiPanelState::load().unwrap_or_else(|| MultiPanelState::open(4.0));
        Self::from_state(state)
    }

    fn from_state(state: MultiPanelState) -> Self {
        Self {
            state,
            left_boost: false,
            right_boost: false,
            top_boost: false,
            bottom_boost: false,
            test_mode: None,
            should_quit: false,
        }
    }

    fn update(&mut self, dt: Duration) {
        self.state.update(dt);

        let mut advance: Option<(u8, u8)> = None;
        if let Some(test) = &mut self.test_mode {
            test.timer += dt;
            if test.timer >= Duration::from_millis(150) {
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
                // Start of a new state: reset to the base state.
                set_state_bits(self, state);
            }
            // Simulate the key for this step.
            self.simulate_key(TEST_KEYS[step as usize]);
        }
    }

    fn start_test_mode(&mut self) {
        if let Some(err) = verify_transition_matrix() {
            eprintln!("Transition matrix mismatch: {}", err);
        } else {
            eprintln!("Transition matrix verified: all 1024 state-key cases pass");
        }
        self.test_mode = Some(TestMode {
            state: 0,
            step: 0,
            timer: Duration::ZERO,
        });
        set_state_bits(self, 0);
    }

    fn stop_test_mode(&mut self) {
        self.test_mode = None;
    }

    /// Simulate a key press as if it came from the event loop.
    pub fn simulate_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('T') => {
                if self.test_mode.is_some() {
                    self.stop_test_mode();
                } else {
                    self.start_test_mode();
                }
            }
            KeyCode::Char('h') => self.state.header.toggle_between(0.0, 1.0),
            KeyCode::Char('l') => self.state.left_sidebar.toggle_between(0.0, 1.0),
            KeyCode::Char('r') => self.state.right_sidebar.toggle_between(0.0, 1.0),
            KeyCode::Char('f') => self.state.footer.toggle_between(0.0, 1.0),
            KeyCode::Char(' ') => self.state.toggle_all(),
            KeyCode::Up => {
                let top_closed = self.state.header.current() < 0.5;
                let bottom_open = self.state.footer.current() > 0.5;
                if top_closed && bottom_open {
                    self.bottom_boost = !self.bottom_boost;
                } else if top_closed {
                    self.state.footer.set_target(1.0);
                } else {
                    self.state.header.toggle_between(0.0, 1.0);
                }
            }
            KeyCode::Down => {
                let bottom_closed = self.state.footer.current() < 0.5;
                let top_open = self.state.header.current() > 0.5;
                if bottom_closed && top_open {
                    self.top_boost = !self.top_boost;
                } else if bottom_closed {
                    self.state.header.set_target(1.0);
                } else {
                    self.state.footer.toggle_between(0.0, 1.0);
                }
            }
            KeyCode::Right => {
                let right_closed = self.state.right_sidebar.current() < 0.5;
                let left_open = self.state.left_sidebar.current() > 0.5;
                if right_closed && left_open {
                    self.left_boost = !self.left_boost;
                } else if right_closed {
                    self.state.left_sidebar.set_target(1.0);
                } else {
                    self.state.right_sidebar.toggle_between(0.0, 1.0);
                }
            }
            KeyCode::Left => {
                let left_closed = self.state.left_sidebar.current() < 0.5;
                let right_open = self.state.right_sidebar.current() > 0.5;
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

    fn left_sidebar_fraction(&self) -> f64 {
        if self.left_boost { 0.50 } else { 0.25 }
    }

    fn right_sidebar_fraction(&self) -> f64 {
        if self.right_boost { 0.50 } else { 0.25 }
    }

    fn header_height(&self) -> u16 {
        if self.top_boost { 10 } else { 5 }
    }

    fn footer_height(&self) -> u16 {
        if self.bottom_boost { 10 } else { 5 }
    }
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let app = App::new();
    let result = run_app(&mut terminal, app);
    // Persist the panel state on exit. Ignore save errors so the terminal is
    // always restored cleanly.
    if let Ok(ref app) = result {
        let _ = app.state.save();
    }
    restore_terminal(&mut terminal)?;
    result.map(|_| ())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<App> {
    let mut last_tick = Instant::now();
    while !app.should_quit {
        let now = Instant::now();
        let dt = now - last_tick;
        last_tick = now;

        app.update(dt);
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.simulate_key(key.code);
        }
    }
    Ok(app)
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let left_fraction = app.left_sidebar_fraction();
    let right_fraction = app.right_sidebar_fraction();
    let header_height = app.header_height();
    let footer_height = app.footer_height();
    let test_label = app.test_mode.as_ref().map(|t| {
        let bits = state_bits(app);
        format!(
            "TEST MODE state {}/256 key {}/{} bits {:08b}",
            t.state + 1,
            t.step + 1,
            TEST_KEYS.len(),
            bits
        )
    });

    let help = "[h/l/r/f] toggle  [Space] all  [↑] close top + boost bottom  [↓] close bottom + boost top  [←] close left + boost right  [→] close right + boost left  [Shift+T] test  [q] quit";

    let widget = MultiPanel::new()
        .header_title("Header")
        .left_title("Left")
        .right_title("Right")
        .main_title("Main")
        .footer_title("Footer")
        .header_content(move |_state, area, buf| {
            // Float at the top of the header.
            let top = Rect::new(area.x, area.y, area.width, 1.min(area.height));
            let text = test_label.clone().unwrap_or_else(|| "↑ top".to_string());
            let p = ratatui::widgets::Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
            ratatui::widgets::Widget::render(p, top, buf);
        })
        .left_content(|_state, area, buf| {
            let p = ratatui::widgets::Paragraph::new("← left")
                .alignment(ratatui::layout::Alignment::Left);
            ratatui::widgets::Widget::render(p, area, buf);
        })
        .right_content(|_state, area, buf| {
            let p = ratatui::widgets::Paragraph::new("right →")
                .alignment(ratatui::layout::Alignment::Right);
            ratatui::widgets::Widget::render(p, area, buf);
        })
        .main_content(move |state, area, buf| {
            let grid = NeighborReportGrid::new(
                left_fraction,
                right_fraction,
                header_height,
                footer_height,
            );
            ratatui::widgets::StatefulWidget::render(grid, area, buf, state);
        })
        .footer_content(move |_state, area, buf| {
            // Float at the bottom of the footer.
            let bottom = Rect::new(
                area.x,
                area.y + area.height.saturating_sub(1),
                area.width,
                1.min(area.height),
            );
            let p = ratatui::widgets::Paragraph::new(help).alignment(ratatui::layout::Alignment::Center);
            ratatui::widgets::Widget::render(p, bottom, buf);
        })
        .header_color(Color::Blue)
        .left_color(Color::Green)
        .right_color(Color::Magenta)
        .footer_color(Color::Yellow)
        .main_color(Color::Reset)
        .header_height(header_height)
        .footer_height(footer_height)
        .left_sidebar_width_fraction(left_fraction)
        .right_sidebar_width_fraction(right_fraction);

    f.render_stateful_widget(widget, area, &mut app.state);
}
