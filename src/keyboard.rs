//! Clickable, interactive virtual keyboard widget for Ratatui.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{StatefulWidget, Widget},
};

use crate::mouse::is_mouse_over;

/// Rows of keys in the keyboard graphic.
fn key_rows() -> &'static [&'static [char]] {
    #[cfg(target_os = "macos")]
    {
        static ROWS: &[&[char]] = &[
            &[
                '\u{E001}', '\u{E002}', '\u{E003}', '\u{E004}', '\u{E005}', '\u{E006}', '\u{E007}',
                '\u{E008}', '\u{E009}', '\u{E00A}', '\u{E00B}', '\u{E00C}',
            ],
            &[
                '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08',
            ],
            &[
                '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '[', ']', '\\',
            ],
            &[
                '\x14', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', '\'', '\n',
            ],
            &[
                '\x1e', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/', '\x1f',
            ],
            &[
                '\x10', '\x11', '\x12', '\u{2318}', ' ', '\u{2318}', '\x12', '\x1b', '\x1c', '\x1d', '↑', '\x1a',
            ],
        ];
        ROWS
    }
    #[cfg(not(target_os = "macos"))]
    {
        static ROWS: &[&[char]] = &[
            &[
                '\u{E001}', '\u{E002}', '\u{E003}', '\u{E004}', '\u{E005}', '\u{E006}', '\u{E007}',
                '\u{E008}', '\u{E009}', '\u{E00A}', '\u{E00B}', '\u{E00C}',
            ],
            &[
                '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08',
            ],
            &[
                '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '[', ']', '\\',
            ],
            &[
                '\x14', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', '\'', '\n',
            ],
            &[
                '\x1e', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/', '\x1f',
            ],
            &['\x11', '\x12', ' ', '\x1b', '\x1c', '\x1d', '\x1a'],
        ];
        ROWS
    }
}

/// Actions produced by the keyboard widget interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardAction {
    Char(char),
    Space,
    Function(u8),
    Backspace,
    Tab,
    Enter,
    Caps,
    Shift,
    Ctrl,
    Alt,
    Esc,
    Left,
    Down,
    Right,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Selected,
    Active,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonTheme {
    pub text: Color,
    pub background: Color,
    pub highlight: Color,
    pub shadow: Color,
}

impl ButtonTheme {
    pub const BLUE: Self = Self {
        text: Color::White, // Color::Rgb(16, 24, 48),
        background: Color::Reset, //Color::Rgb(48, 72, 144),
        highlight: Color::White, //Color::Rgb(64, 96, 192),
        shadow: Color::Rgb(32, 48, 96),
    };
}

#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: Line<'a>,
    theme: ButtonTheme,
    state: ButtonState,
}

impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Self {
            label: label.into(),
            theme: ButtonTheme::BLUE,
            state: ButtonState::Normal,
        }
    }

    pub const fn theme(mut self, theme: ButtonTheme) -> Self {
        self.theme = theme;
        self
    }

    pub const fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    const fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            ButtonState::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            ButtonState::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Active => (theme.background, Color::White, theme.highlight, theme.shadow),
        }
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{Block, BorderType, Borders};
        let (background, text, _shadow, highlight) = self.colors();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(highlight).bg(background));
        let inner = block.inner(area);
        buf.set_style(area, Style::new().bg(background).fg(text));
        ratatui::widgets::Widget::render(block, area, buf);
        if inner.width > 0 && inner.height > 0 {
            let label = self.label.style(Style::new().fg(text));
            buf.set_line(
                inner.x + (inner.width.saturating_sub(label.width() as u16)) / 2,
                inner.y + (inner.height.saturating_sub(1)) / 2,
                &label,
                inner.width,
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardState {
    pub button_states: Vec<ButtonState>,
    pub last_areas: Vec<Option<Rect>>,
    pub keys: Vec<char>,
    pub last_output: String,
    pub caps_lock: bool,
    pub last_pressed: Option<char>,
    pub last_pressed_at: Option<std::time::Instant>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardState {
    pub fn new() -> Self {
        let mut keys = Vec::new();
        for row in key_rows() {
            keys.extend_from_slice(row);
        }
        Self {
            button_states: vec![ButtonState::default(); keys.len()],
            last_areas: vec![None; keys.len()],
            keys,
            last_output: String::new(),
            caps_lock: false,
            last_pressed: None,
            last_pressed_at: None,
        }
    }

    /// Highlight the key matching `code` (used when a real key is pressed).
    pub fn press_key(&mut self, code: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;
        for bs in &mut self.button_states {
            *bs = ButtonState::Normal;
        }
        let target = match code {
            KeyCode::Char(c) => c,
            KeyCode::Enter => '\n',
            KeyCode::Tab => '\t',
            KeyCode::Backspace => '\x08',
            KeyCode::Esc => '\x1b',
            KeyCode::Left => '\x1c',
            KeyCode::Down => '\x1d',
            KeyCode::Right => '\x1a',
            KeyCode::Up => '↑',
            KeyCode::F(n) => char::from_u32(0xE000 + n as u32).unwrap_or(' '),
            _ => return,
        };
        self.last_pressed = Some(target);
        self.last_pressed_at = Some(std::time::Instant::now());
        for (i, &k) in self.keys.iter().enumerate() {
            if k == target || k.to_ascii_lowercase() == target.to_ascii_lowercase() {
                if let Some(bs) = self.button_states.get_mut(i) {
                    *bs = ButtonState::Active;
                }
                break;
            }
        }
    }

    pub fn handle_mouse(&mut self, event: MouseEvent) -> Option<KeyboardAction> {
        let mut clicked: Option<usize> = None;

        for (i, area) in self.last_areas.iter().enumerate() {
            if let Some(area) = area {
                if is_mouse_over(*area, event.column, event.row)
                    && matches!(event.kind, MouseEventKind::Down(MouseButton::Left))
                {
                    clicked = Some(i);
                }
            }
        }

        // update hover states
        for (i, bs) in self.button_states.iter_mut().enumerate() {
            if let Some(area) = self.last_areas[i] {
                let over = is_mouse_over(area, event.column, event.row);
                match event.kind {
                    MouseEventKind::Moved => {
                        if over && *bs != ButtonState::Active {
                            *bs = ButtonState::Selected;
                        } else if !over && *bs == ButtonState::Selected {
                            *bs = ButtonState::Normal;
                        }
                    }
                    MouseEventKind::Down(MouseButton::Left) if over => {
                        *bs = if *bs == ButtonState::Active {
                            ButtonState::Normal
                        } else {
                            ButtonState::Active
                        };
                    }
                    _ => {}
                }
            }
        }

        if let Some(i) = clicked {
            if let Some(&key) = self.keys.get(i) {
                if key == '\x14' {
                    self.caps_lock = !self.caps_lock;
                }
                let action = match key {
                    ' ' => KeyboardAction::Space,
                    '\x08' => KeyboardAction::Backspace,
                    '\x10' => return None, // Fn: no terminal action
                    '\u{2318}' => return None, // Cmd: no terminal action
                    '\t' => KeyboardAction::Tab,
                    '\n' => KeyboardAction::Enter,
                    '\x14' => KeyboardAction::Caps,
                    '\x1e' | '\x1f' => KeyboardAction::Shift,
                    '\x11' => KeyboardAction::Ctrl,
                    '\x12' => KeyboardAction::Alt,
                    '\x1b' => KeyboardAction::Esc,
                    '\x1c' => KeyboardAction::Left,
                    '\x1d' => KeyboardAction::Down,
                    '↑' => KeyboardAction::Up,
                    '\x1a' => KeyboardAction::Right,
                    c if (c as u32) >= 0xE001 && (c as u32) <= 0xE00C => {
                        KeyboardAction::Function(((c as u32) - 0xE000) as u8)
                    }
                    _ => {
                        if key.is_alphabetic() && !self.caps_lock {
                            KeyboardAction::Char(key.to_ascii_lowercase())
                        } else {
                            KeyboardAction::Char(key)
                        }
                    }
                };
                let label = match key {
                    ' ' => "Space".to_string(),
                    '\x08' => "Backspace".to_string(),
                    '\t' => "Tab".to_string(),
                    '\n' => "Enter".to_string(),
                    '\x14' => "Caps".to_string(),
                    '\x1e' | '\x1f' => "Shift".to_string(),
                    '\x11' => "Ctrl".to_string(),
                    '\x12' => "Alt".to_string(),
                    '\x10' => "Fn".to_string(),
                    '\x1b' => "Esc".to_string(),
                    '\x1c' => "Left".to_string(),
                    '\x1d' => "Down".to_string(),
                    '↑' => "Up".to_string(),
                    '\x1a' => "Right".to_string(),
                    c if (c as u32) >= 0xE001 && (c as u32) <= 0xE00C => {
                        format!("F{}", (c as u32) - 0xE000)
                    }
                    '\u{2318}' => "Cmd".to_string(),
                    '\u{2325}' => "Opt".to_string(),
                    _ => key.to_string(),
                };
                self.last_output = format!("Clicked: {}", label);
                return Some(action);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardWidget {
    pub rows: Vec<Vec<char>>,
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self {
            rows: key_rows().iter().map(|r| r.to_vec()).collect(),
        }
    }
}

impl StatefulWidget for KeyboardWidget {
    type State = KeyboardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut y = area.y;
        let mut idx = 0usize;

        // Expire the highlight after 200ms.
        let expired = state
            .last_pressed_at
            .map(|t| t.elapsed().as_millis() > 200)
            .unwrap_or(true);
        if expired {
            for bs in &mut state.button_states {
                *bs = ButtonState::Normal;
            }
            state.last_pressed = None;
            state.last_pressed_at = None;
        } else if let Some(pressed) = state.last_pressed {
            for bs in &mut state.button_states {
                *bs = ButtonState::Normal;
            }
            for (i, &k) in state.keys.iter().enumerate() {
                if k == pressed || k.to_ascii_lowercase() == pressed.to_ascii_lowercase() {
                    if let Some(bs) = state.button_states.get_mut(i) {
                        *bs = ButtonState::Active;
                    }
                    break;
                }
            }
        }

        // Responsive breakpoints for the keyboard widget.
        //   61×43 -> smallest size that still shows key characters.
        //   73×31 -> smallest size that shows keys (outlines only).
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Mode {
            Hidden,
            Unlabeled,
            Labeled,
        }

        let mode = if area.width >= 61 && area.height >= 43 {
            Mode::Labeled
        } else if area.width >= 73 && area.height >= 31 {
            Mode::Unlabeled
        } else {
            Mode::Hidden
        };

        if mode == Mode::Hidden {
            for slot in &mut state.last_areas {
                *slot = None;
            }
            return;
        }

        // Base dimensions depend on the responsive mode.
        let (base_key_width, base_space_width, base_key_height) = match mode {
            Mode::Labeled => (3, 9, 3),
            Mode::Unlabeled => (4, 12, 2),
            Mode::Hidden => unreachable!(),
        };
        const BASE_GAP: u16 = 1;

        // Compute the width of the widest row at 1x scale.
        let max_base_row_width: u16 = self
            .rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|k| if *k == ' ' { base_space_width } else { base_key_width })
                    .sum::<u16>()
                    .saturating_add((row.len() as u16).saturating_sub(1))
            })
            .max()
            .unwrap_or(0);

        // Total height at 1x scale: rows of keys plus gaps between them.
        let total_base_height: u16 = (self.rows.len() as u16)
            .saturating_mul(base_key_height + BASE_GAP)
            .saturating_sub(BASE_GAP);

        // Pick a single global scale that fits both the width and the height.
        let scale_x = if max_base_row_width > 0 {
            area.width / max_base_row_width
        } else {
            1
        };
        let scale_y = if total_base_height > 0 {
            area.height / total_base_height
        } else {
            1
        };
        let scale = scale_x.min(scale_y).max(1);

        // If the chosen scale still doesn't fit vertically (e.g. the allocated
        // area is shorter than the 1x keyboard), switch to a compact layout
        // that removes vertical gaps and distributes the height evenly.
        let scaled_total_height = total_base_height.saturating_mul(scale);
        let rows = self.rows.len() as u16;
        let (key_height, v_gap) = if scaled_total_height <= area.height {
            (
                base_key_height.saturating_mul(scale),
                BASE_GAP.saturating_mul(scale),
            )
        } else {
            let h = (area.height / rows).max(1);
            let gap = if h.saturating_mul(rows)
                .saturating_add(rows.saturating_sub(1))
                <= area.height
            {
                1
            } else {
                0
            };
            (h, gap)
        };

        let h_gap = scale.max(1);

        for row in &self.rows {
            let base_row_width: u16 = row
                .iter()
                .map(|k| if *k == ' ' { base_space_width } else { base_key_width })
                .sum::<u16>()
                .saturating_add((row.len() as u16).saturating_sub(1));
            let row_width = base_row_width.saturating_mul(scale);
            let indent = area.width.saturating_sub(row_width) / 2;
            let mut x = area.x + indent;

            for key in row {
                let label = if mode == Mode::Unlabeled {
                    "".to_string()
                } else {
                    match *key {
                        ' ' => "Space".to_string(),
                        '\x08' => "Bksp".to_string(),
                        '\t' => "Tab".to_string(),
                        '\n' => "Enter".to_string(),
                        '\x14' => "Caps".to_string(),
                        '\x1e' | '\x1f' => "Shift".to_string(),
                        '\x11' => "Ctrl".to_string(),
                        '\x12' => "Alt".to_string(),
                        '\x10' => "Fn".to_string(),
                        '\x1b' => "Esc".to_string(),
                        '\x1c' => "←".to_string(),
                        '\x1d' => "↓".to_string(),
                        '\x1a' => "→".to_string(),
                        c if (c as u32) >= 0xE001 && (c as u32) <= 0xE00C => {
                            format!("F{}", (c as u32) - 0xE000)
                        }
                        _ => {
                            if key.is_alphabetic() {
                                if state.caps_lock {
                                    key.to_uppercase().to_string()
                                } else {
                                    key.to_lowercase().to_string()
                                }
                            } else {
                                key.to_string()
                            }
                        }
                    }
                };
                let base_w: u16 = if *key == ' ' { base_space_width } else { base_key_width };
                let w = base_w.saturating_mul(scale).max(base_w);
                let btn_area = Rect::new(x, y, w, key_height);

                if x + w <= area.x + area.width && y + key_height <= area.y + area.height {
                    let st = state
                        .button_states
                        .get(idx)
                        .copied()
                        .unwrap_or(ButtonState::Normal);
                    let button = Button::new(label).state(st);
                    if let Some(slot) = state.last_areas.get_mut(idx) {
                        *slot = Some(btn_area);
                    }
                    button.render(btn_area, buf);
                }
                idx += 1;
                x += w + h_gap;
            }
            y += key_height + v_gap;
            if y >= area.y + area.height {
                break;
            }
        }
    }
}
