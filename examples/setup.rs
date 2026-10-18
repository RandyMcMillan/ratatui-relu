use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let project_dir = "keyboard_demo";
    let src_dir = format!("{}/src", project_dir);

    println!("[*] Creating directory structure for '{}'...", project_dir);
    fs::create_dir_all(&src_dir)?;

    // 1. Write Cargo.toml
    let cargo_toml_content = r#"[package]
name = "keyboard_demo"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.30.2"
crossterm = "0.29"
ratatui-relu = { path = "../" }
"#;

    let cargo_toml_path = format!("{}/Cargo.toml", project_dir);
    println!("[*] Writing {}", cargo_toml_path);
    fs::write(&cargo_toml_path, cargo_toml_content)?;

    // 2. Write src/main.rs
    let main_rs_content = r#"//! Virtual Keyboard widget demo.
//!
//! Run with: cargo run

use std::io::{self, Stdout, stdout};
use std::time::Instant;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_relu::keyboard::{KeyboardState, KeyboardWidget, KeyboardAction};
use ratatui_relu::mouse::{disable_mouse_capture, enable_mouse_capture};

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
    
    // Initialize keyboard state and widget
    let mut keyboard_state = KeyboardState::new();
    let keyboard_widget = KeyboardWidget::default();
    
    let mut last_output = String::from("Click keys on the virtual keyboard below!");
    let mut should_quit = false;

    let mut last_tick = Instant::now();
    while !should_quit {
        let _ = last_tick;
        last_tick = Instant::now();

        terminal.draw(|f| {
            let area = f.area();
            
            // Layout: Top for output display, bottom for keyboard widget
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(15)])
                .split(area);

            // Render output display box
            let output_paragraph = Paragraph::new(last_output.clone())
                .block(Block::default().title("Output / Action").borders(Borders::ALL))
                .alignment(Alignment::Center);
            f.render_widget(output_paragraph, chunks[0]);

            // Render stateful keyboard widget
            let keyboard_block = Block::default()
                .title("Virtual Keyboard [Press Esc or 'q' to quit]")
                .borders(Borders::ALL);
            let inner_keyboard_area = keyboard_block.inner(chunks[1]);
            f.render_widget(keyboard_block, chunks[1]);

            f.render_stateful_widget(
                keyboard_widget.clone(),
                inner_keyboard_area,
                &mut keyboard_state,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        should_quit = true;
                    }
                }
                Event::Mouse(mouse) => {
                    // Pass mouse events to the keyboard state to handle hover/clicks
                    if let Some(action) = keyboard_state.handle_mouse(mouse) {
                        match action {
                            KeyboardAction::Char(c) => {
                                last_output = format!("Clicked Character: {}", c);
                            }
                            KeyboardAction::Space => {
                                last_output = String::from("Clicked Character: [Space]");
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    restore_terminal(&mut terminal)?;
    println!("App exited cleanly.");
    Ok(())
}
"#;

    let main_rs_path = format!("{}/src/main.rs", project_dir);
    println!("[*] Writing {}", main_rs_path);
    fs::write(&main_rs_path, main_rs_content)?;

    println!(
        "\n[+] Success! Project generated in directory: './{}'",
        project_dir
    );
    println!("[+] To run your app:");
    println!("    cd keyboard_demo");
    println!("    cargo run");

    Ok(())
}
