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
    permutation: u8,
    timer: Duration,
}

impl App {
    fn new() -> Self {
        // Try to restore previous panel state; fall back to all panels open.
        let state = MultiPanelState::load().unwrap_or_else(|| MultiPanelState::open(4.0));
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

        let mut next: Option<u8> = None;
        if let Some(test) = &mut self.test_mode {
            test.timer += dt;
            if test.timer >= Duration::from_millis(400) {
                test.timer = Duration::ZERO;
                test.permutation = (test.permutation + 1) % 16;
                next = Some(test.permutation);
            }
        }
        if let Some(p) = next {
            self.apply_permutation(p);
        }
    }

    fn start_test_mode(&mut self) {
        self.test_mode = Some(TestMode {
            permutation: 0,
            timer: Duration::ZERO,
        });
        self.apply_permutation(0);
    }

    fn stop_test_mode(&mut self) {
        self.test_mode = None;
    }

    fn apply_permutation(&mut self, permutation: u8) {
        // bit 0 -> header, 1 -> left, 2 -> right, 3 -> footer
        let open = |bit: u8| if (permutation >> bit) & 1 == 1 { 1.0 } else { 0.0 };
        self.state.header.set_target(open(0));
        self.state.left_sidebar.set_target(open(1));
        self.state.right_sidebar.set_target(open(2));
        self.state.footer.set_target(open(3));
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
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Char('T') => {
                    if app.test_mode.is_some() {
                        app.stop_test_mode();
                    } else {
                        app.start_test_mode();
                    }
                }
                KeyCode::Char('h') => app.state.header.toggle_between(0.0, 1.0),
                KeyCode::Char('l') => app.state.left_sidebar.toggle_between(0.0, 1.0),
                KeyCode::Char('r') => app.state.right_sidebar.toggle_between(0.0, 1.0),
                KeyCode::Char('f') => app.state.footer.toggle_between(0.0, 1.0),
                KeyCode::Char(' ') => app.state.toggle_all(),
                KeyCode::Up => {
                    let top_closed = app.state.header.current() < 0.5;
                    let bottom_open = app.state.footer.current() > 0.5;
                    if top_closed && bottom_open {
                        app.bottom_boost = !app.bottom_boost;
                    } else if top_closed {
                        app.state.footer.set_target(1.0);
                    } else {
                        app.state.header.toggle_between(0.0, 1.0);
                    }
                }
                KeyCode::Down => {
                    let bottom_closed = app.state.footer.current() < 0.5;
                    let top_open = app.state.header.current() > 0.5;
                    if bottom_closed && top_open {
                        app.top_boost = !app.top_boost;
                    } else if bottom_closed {
                        app.state.header.set_target(1.0);
                    } else {
                        app.state.footer.toggle_between(0.0, 1.0);
                    }
                }
                KeyCode::Right => {
                    let right_closed = app.state.right_sidebar.current() < 0.5;
                    let left_open = app.state.left_sidebar.current() > 0.5;
                    if right_closed && left_open {
                        app.left_boost = !app.left_boost;
                    } else if right_closed {
                        app.state.left_sidebar.set_target(1.0);
                    } else {
                        app.state.right_sidebar.toggle_between(0.0, 1.0);
                    }
                }
                KeyCode::Left => {
                    let left_closed = app.state.left_sidebar.current() < 0.5;
                    let right_open = app.state.right_sidebar.current() > 0.5;
                    if left_closed && right_open {
                        app.right_boost = !app.right_boost;
                    } else if left_closed {
                        app.state.right_sidebar.set_target(1.0);
                    } else {
                        app.state.left_sidebar.toggle_between(0.0, 1.0);
                    }
                }
                _ => {}
            }
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
    let test_label = app
        .test_mode
        .as_ref()
        .map(|t| format!("TEST MODE permutation {}/16", t.permutation + 1));

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
