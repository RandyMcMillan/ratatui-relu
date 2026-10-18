//! Multi-panel layout controlled by independent animated transitions.
//!
//! Demonstrates the reusable [`MultiPanelApp`] from `ratatui_relu::multi_panel`.
//!
//! Run in a real terminal with: cargo run

use std::io::{self, Stdout, stdout};
use std::time::Instant;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use ratatui_relu::{
    mouse::{disable_mouse_capture, enable_mouse_capture},
    multi_panel::MultiPanelApp,
};

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    enable_mouse_capture()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_mouse_capture()?;
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = MultiPanelApp::new();
    let result = run_app(&mut terminal, &mut app);
    // Persist the panel state on exit. Ignore save errors so the terminal is
    // always restored cleanly.
    let _ = app.save();
    restore_terminal(&mut terminal)?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut MultiPanelApp,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        let now = Instant::now();
        let dt = now - last_tick;
        last_tick = now;

        app.update(dt);
        terminal.draw(|f| app.render(f))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.footer_terminal_focus {
                        if key.code == crossterm::event::KeyCode::Esc {
                            app.footer_terminal_focus = false;
                        } else if key.code == crossterm::event::KeyCode::Up {
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.scroll_up(5);
                            }
                        } else if key.code == crossterm::event::KeyCode::Down {
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.scroll_down(5);
                            }
                        } else if key.code == crossterm::event::KeyCode::PageUp {
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.scroll_up(10);
                            }
                        } else if key.code == crossterm::event::KeyCode::PageDown {
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.scroll_down(10);
                            }
                        } else {
                            let bytes = match key.code {
                                crossterm::event::KeyCode::Char(c) => {
                                    if key
                                        .modifiers
                                        .contains(crossterm::event::KeyModifiers::CONTROL)
                                    {
                                        vec![c as u8 & 0x1f]
                                    } else {
                                        c.to_string().into_bytes()
                                    }
                                }
                                crossterm::event::KeyCode::Enter => vec![b'\n'],
                                crossterm::event::KeyCode::Backspace => vec![0x7f],
                                crossterm::event::KeyCode::Tab => vec![b'\t'],
                                _ => vec![],
                            };
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.write_input(&bytes);
                            }
                        }
                    } else if app.show_terminal {
                        if key.code == crossterm::event::KeyCode::Esc {
                            app.show_terminal = false;
                        } else if key.code == crossterm::event::KeyCode::PageUp {
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.scroll_up(10);
                            }
                        } else if key.code == crossterm::event::KeyCode::PageDown {
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.scroll_down(10);
                            }
                        } else {
                            // Send keystroke to terminal.
                            let bytes = match key.code {
                                crossterm::event::KeyCode::Char(c) => {
                                    if key
                                        .modifiers
                                        .contains(crossterm::event::KeyModifiers::CONTROL)
                                    {
                                        vec![c as u8 & 0x1f]
                                    } else {
                                        c.to_string().into_bytes()
                                    }
                                }
                                crossterm::event::KeyCode::Enter => vec![b'\n'],
                                crossterm::event::KeyCode::Backspace => vec![0x7f],
                                crossterm::event::KeyCode::Tab => vec![b'\t'],
                                _ => vec![],
                            };
                            if let Some(term) = &mut *app.terminal.borrow_mut() {
                                term.write_input(&bytes);
                            }
                        }
                    } else if key.code == crossterm::event::KeyCode::Char('e') {
                        if let Some(path) = app.selected_file.clone() {
                            // Suspend the TUI and launch the user's editor.
                            disable_mouse_capture()?;
                            disable_raw_mode()?;
                            terminal.backend_mut().execute(LeaveAlternateScreen)?;
                            let editor =
                                std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                            let _ = std::process::Command::new(&editor).arg(&path).status();
                            // Restore the TUI.
                            stdout().execute(EnterAlternateScreen)?;
                            enable_raw_mode()?;
                            enable_mouse_capture()?;
                            terminal.clear()?;
                            // Reload the preview after editing.
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                app.file_preview = content;
                            }
                        }
                    } else if key.code == crossterm::event::KeyCode::Esc {
                        // Back out of everything without quitting.
                        app.show_help = false;
                        app.show_settings = false;
                        app.show_terminal = false;
                        if app.right_full {
                            app.right_full = false;
                        } else if app.right_boost {
                            app.right_boost = false;
                        } else if app.state.right_sidebar.target() > 0.5 {
                            app.state.right_sidebar.set_target(0.0);
                        } else if app.main_boost {
                            app.main_boost = false;
                        } else {
                            app.main_active = false;
                        }
                        app.file_tree_focus = false;
                    } else if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT)
                        && matches!(
                            key.code,
                            crossterm::event::KeyCode::Char('Q')
                                | crossterm::event::KeyCode::Char('P')
                        )
                    {
                        break;
                    } else {
                        app.handle_key_event(key);
                        // Esc does not quit the app; it only closes overlays/deactivates views.
                    }
                    // Repaint immediately after a key press so state changes
                    // are visible right away.
                    terminal.draw(|f| app.render(f))?;
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
