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

struct App {
    state: MultiPanelState,
    right_boost: bool,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        // Try to restore previous panel state; fall back to all panels open.
        let state = MultiPanelState::load().unwrap_or_else(|| MultiPanelState::open(4.0));
        Self {
            state,
            right_boost: false,
            should_quit: false,
        }
    }

    fn update(&mut self, dt: Duration) {
        self.state.update(dt);
    }

    fn right_sidebar_fraction(&self) -> f64 {
        if self.right_boost {
            0.50
        } else {
            0.25
        }
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
                KeyCode::Char('h') => app.state.header.toggle_between(0.0, 1.0),
                KeyCode::Char('l') => app.state.left_sidebar.toggle_between(0.0, 1.0),
                KeyCode::Char('r') => app.state.right_sidebar.toggle_between(0.0, 1.0),
                KeyCode::Char('f') => app.state.footer.toggle_between(0.0, 1.0),
                KeyCode::Char(' ') => app.state.toggle_all(),
                KeyCode::Up => {
                    app.state.header.set_target(0.0);
                    app.state.footer.set_target(1.0);
                }
                KeyCode::Down => {
                    app.state.footer.set_target(0.0);
                    app.state.header.set_target(1.0);
                }
                KeyCode::Right => {
                    if app.state.right_sidebar.current() < 0.5 {
                        app.state.left_sidebar.set_target(1.0);
                    } else {
                        app.state.right_sidebar.toggle_between(0.0, 1.0);
                    }
                }
                KeyCode::Left => {
                    let left_closed = app.state.left_sidebar.current() < 0.5;
                    let right_open = app.state.right_sidebar.current() > 0.5;
                    if left_closed && right_open {
                        // Left is closed and right is open: double the right panel width.
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

    let help = "[h/l/r/f] toggle  [Space] all  [↑] close header + open footer  [↓] close footer + open header  [←] toggle left / open right if left closed  [→] toggle right / open left if right closed  [q] quit";

    let widget = MultiPanel::new()
        .header_title("Header")
        .left_title("Left")
        .right_title("Right")
        .main_title("Main")
        .footer_title("Footer")
        .header_content(|_state, area, buf| {
            // Float at the top of the header.
            let top = Rect::new(area.x, area.y, area.width, 1.min(area.height));
            let p = ratatui::widgets::Paragraph::new("↑ top").alignment(ratatui::layout::Alignment::Center);
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
        .main_content(|state, area, buf| {
            let left_fraction = 0.25;
            let right_fraction = app.right_sidebar_fraction();

            // Split the main area into a 3x3 grid of sub-widgets.
            let rows = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                ])
                .split(area);

            let cols_top = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                ])
                .split(rows[0]);
            let cols_mid = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                ])
                .split(rows[1]);
            let cols_bot = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                    ratatui::layout::Constraint::Ratio(1, 3),
                ])
                .split(rows[2]);

            fn subwidget(title: &str, value: String, area: Rect, buf: &mut ratatui::buffer::Buffer) {
                let block = ratatui::widgets::Block::default()
                    .title(title)
                    .borders(ratatui::widgets::Borders::ALL);
                let inner = block.inner(area);
                ratatui::widgets::Widget::render(block, area, buf);
                if inner.width > 0 && inner.height > 0 {
                    let p = ratatui::widgets::Paragraph::new(value)
                        .alignment(ratatui::layout::Alignment::Center);
                    ratatui::widgets::Widget::render(p, inner, buf);
                }
            }

            subwidget("Top-Left", format!("{}x{}", cols_top[0].width, cols_top[0].height), cols_top[0], buf);
            subwidget(
                "Top",
                format!("{} rows", state.header_height(5)),
                cols_top[1],
                buf,
            );
            subwidget("Top-Right", format!("{}x{}", cols_top[2].width, cols_top[2].height), cols_top[2], buf);

            subwidget(
                "Left",
                format!("{} cols", state.left_sidebar_width(area.width, left_fraction)),
                cols_mid[0],
                buf,
            );
            subwidget("Center", format!("{}x{}", area.width, area.height), cols_mid[1], buf);
            subwidget(
                "Right",
                format!("{} cols", state.right_sidebar_width(area.width, right_fraction)),
                cols_mid[2],
                buf,
            );

            subwidget("Bot-Left", format!("{}x{}", cols_bot[0].width, cols_bot[0].height), cols_bot[0], buf);
            subwidget(
                "Bottom",
                format!("{} rows", state.footer_height(5)),
                cols_bot[1],
                buf,
            );
            subwidget("Bot-Right", format!("{}x{}", cols_bot[2].width, cols_bot[2].height), cols_bot[2], buf);
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
        .panel_height(5)
        .left_sidebar_width_fraction(0.25)
        .right_sidebar_width_fraction(app.right_sidebar_fraction());

    f.render_stateful_widget(widget, area, &mut app.state);
}
