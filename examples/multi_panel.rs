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
    style::Color,
    Terminal,
};

use ratarelu::multi_panel::{MultiPanel, MultiPanelState};

struct App {
    state: MultiPanelState,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            state: MultiPanelState::open(4.0),
            should_quit: false,
        }
    }

    fn update(&mut self, dt: Duration) {
        self.state.update(dt);
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
    restore_terminal(&mut terminal)?;
    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<()> {
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
                    if app.state.left_sidebar.current() < 0.5 {
                        app.state.right_sidebar.set_target(1.0);
                    } else {
                        app.state.left_sidebar.toggle_between(0.0, 1.0);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
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
        .footer_content(help)
        .header_color(Color::Blue)
        .left_color(Color::Green)
        .right_color(Color::Magenta)
        .footer_color(Color::Yellow)
        .main_color(Color::Reset)
        .panel_height(5)
        .sidebar_width_fraction(0.25);

    f.render_stateful_widget(widget, area, &mut app.state);
}
