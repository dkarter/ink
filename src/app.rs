//! Interactive prompt event loop.

use std::{
    fmt::Write as _,
    io::{self, Write},
};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    cli::{PromptKind, PromptRuntimeOptions, ResolvedPromptOptions},
    config::StartupMode,
    editor::{Editor, Mode},
    terminal::{CancelReason, CursorShape, PromptOutcome, TerminalSession},
    ui::{CursorRequest, Input, InputState, Textarea, TextareaState},
};

pub(crate) fn run(
    session: &mut TerminalSession,
    kind: PromptKind,
    seed: &str,
    prompt: &PromptRuntimeOptions,
    options: ResolvedPromptOptions,
) -> io::Result<PromptOutcome> {
    let seed = match kind {
        PromptKind::Input => single_line(seed),
        PromptKind::Textarea => seed.to_owned(),
    };
    let mut editor = match kind {
        PromptKind::Input => Editor::input(seed).expect("input seed is normalized"),
        PromptKind::Textarea => Editor::textarea(seed),
    };
    if options.settings.startup_mode == StartupMode::Insert {
        editor.enter_insert();
        editor.set_cursor(crate::editor::Position::new(usize::MAX, usize::MAX));
    }

    let compact_height = match kind {
        PromptKind::Input => Some(3),
        PromptKind::Textarea if !prompt.fullscreen => Some(6),
        PromptKind::Textarea => None,
    };
    let (viewport, mut prompt_area, mut input_row) = match compact_height {
        Some(height) => {
            reserve_rows(session, height)?;
            let (_, row) = session.cursor_position()?;
            let (width, height) = crossterm::terminal::size()?;
            let area = compact_area(row, width, height, compact_height.unwrap());
            (Viewport::Fixed(area), area, row)
        }
        None => {
            let (width, height) = crossterm::terminal::size()?;
            (Viewport::Fullscreen, Rect::new(0, 0, width, height), 0)
        }
    };
    let backend = CrosstermBackend::new(SessionWriter(session));
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;
    let mut input_state = InputState::default();
    let mut textarea_state = TextareaState::default();
    let mut cursor_style = None;

    loop {
        let mut cursor = None;
        terminal.draw(|frame| {
            let area = frame.area();
            match kind {
                PromptKind::Input => {
                    let input_area = Rect::new(area.x, area.y, area.width, area.height.min(1));
                    Input::new(&editor)
                        .prompt(&prompt.prompt)
                        .palette(options.theme.palette)
                        .show_mode(false)
                        .render(input_area, frame.buffer_mut(), &mut input_state);
                    render_input_status(frame.buffer_mut(), area, editor.mode(), options);
                    cursor = input_state.cursor();
                }
                PromptKind::Textarea => {
                    Textarea::new(&editor)
                        .palette(options.theme.palette)
                        .hint("ctrl-d submit  ctrl-c cancel")
                        .render(area, frame.buffer_mut(), &mut textarea_state);
                    cursor = textarea_state.cursor();
                }
            }
            if let Some(request) = cursor {
                frame.set_cursor_position(request.position);
            }
        })?;
        if cursor.map(|request| request.style) != cursor_style {
            apply_cursor(session, cursor)?;
            session.flush()?;
            cursor_style = cursor.map(|request| request.style);
        }

        match session.read_event()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if let Some(outcome) = handle_key(&mut editor, kind, key) {
                    clear_prompt(session, prompt_area)?;
                    return Ok(outcome);
                }
            }
            Event::Paste(value) if editor.mode() == Mode::Insert => {
                let value = if kind == PromptKind::Input {
                    single_line(&value)
                } else {
                    value.replace("\r\n", "\n")
                };
                editor.insert(&value);
            }
            Event::Resize(width, height) if compact_height.is_some() => {
                input_row = input_row.min(height.saturating_sub(1));
                prompt_area = compact_area(input_row, width, height, compact_height.unwrap());
                terminal.resize(prompt_area)?;
            }
            Event::Resize(width, height) => {
                prompt_area = Rect::new(0, 0, width, height);
            }
            _ => {}
        }
    }
}

fn compact_area(row: u16, width: u16, height: u16, preferred_height: u16) -> Rect {
    Rect::new(
        0,
        row.min(height.saturating_sub(1)),
        width,
        preferred_height.min(height.saturating_sub(row)),
    )
}

fn reserve_rows(session: &TerminalSession, height: u16) -> io::Result<()> {
    let lines = height.saturating_sub(1);
    for _ in 0..lines {
        session.write_ui(b"\r\n")?;
    }
    if lines > 0 {
        session.write_ui(format!("\x1b[{lines}A").as_bytes())?;
    }
    Ok(())
}

fn clear_prompt(session: &TerminalSession, area: Rect) -> io::Result<()> {
    let mut erase = String::with_capacity(usize::from(area.height) * 12);
    for row in area.y..area.bottom() {
        write!(erase, "\x1b[{};{}H\x1b[2K", row + 1, area.x + 1)
            .expect("writing to a string cannot fail");
    }
    session.write_ui(erase.as_bytes())?;
    session.flush()
}

fn render_input_status(
    buffer: &mut ratatui::buffer::Buffer,
    area: Rect,
    mode: Mode,
    options: ResolvedPromptOptions,
) {
    if area.height < 2 || area.width == 0 {
        return;
    }
    let palette = options.theme.palette;
    let status_area = Rect::new(area.x, area.bottom() - 1, area.width, 1);
    let mode_color = match mode {
        Mode::Insert => palette.insert_mode,
        Mode::Normal => palette.normal_mode,
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => palette.visual_mode,
    };
    let mode_width = crate::ui::mode_label(mode).len() + 2;
    let hint = if usize::from(area.width) > mode_width + "enter submit  ctrl-c cancel".len() {
        "enter submit  ctrl-c cancel"
    } else if usize::from(area.width) > mode_width + "enter submit".len() {
        "enter submit"
    } else {
        ""
    };
    Paragraph::new(Line::from(Span::styled(
        hint,
        Style::default().fg(palette.muted.into()),
    )))
    .alignment(Alignment::Right)
    .render(status_area, buffer);
    Paragraph::new(Line::from(Span::styled(
        format!(" {} ", crate::ui::mode_label(mode)),
        Style::default()
            .fg(palette.background.into())
            .bg(mode_color.into()),
    )))
    .render(status_area, buffer);
}

fn handle_key(editor: &mut Editor, kind: PromptKind, key: KeyEvent) -> Option<PromptOutcome> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Some(PromptOutcome::Cancelled(CancelReason::Interrupt)),
            KeyCode::Char('d') => Some(PromptOutcome::Accepted(editor.text().to_owned())),
            KeyCode::Char('v') if editor.mode() == Mode::Normal => {
                editor.enter_visual_block();
                None
            }
            _ => None,
        };
    }

    match editor.mode() {
        Mode::Insert => match key.code {
            KeyCode::Esc => editor.escape(),
            KeyCode::Left => editor.move_left(),
            KeyCode::Right => editor.move_right(),
            KeyCode::Up => editor.move_up(),
            KeyCode::Down => editor.move_down(),
            KeyCode::Home => editor.move_to_line_start(),
            KeyCode::End => editor.move_to_line_end(),
            KeyCode::Backspace => {
                editor.backspace();
            }
            KeyCode::Enter if kind == PromptKind::Input => {
                return Some(PromptOutcome::Accepted(editor.text().to_owned()));
            }
            KeyCode::Enter => {
                editor.insert("\n");
            }
            KeyCode::Tab => {
                editor.insert("\t");
            }
            KeyCode::Char(character) => {
                editor.insert(&character.to_string());
            }
            _ => {}
        },
        Mode::Normal => match key.code {
            KeyCode::Char('q') => return Some(PromptOutcome::Cancelled(CancelReason::NormalQuit)),
            KeyCode::Enter => return Some(PromptOutcome::Accepted(editor.text().to_owned())),
            KeyCode::Char('i') => editor.enter_insert(),
            KeyCode::Char('v') => editor.enter_visual(),
            KeyCode::Char('V') => editor.enter_visual_line(),
            KeyCode::Char('x') => {
                editor.delete_at_cursor();
            }
            code => move_cursor(editor, code),
        },
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => match key.code {
            KeyCode::Esc => editor.escape(),
            KeyCode::Char('d') | KeyCode::Char('x') => {
                editor.delete_selection();
            }
            KeyCode::Char('c') => {
                editor.change_selection();
            }
            KeyCode::Char('y') => {
                editor.yank_selection();
            }
            code => move_cursor(editor, code),
        },
    }
    None
}

fn move_cursor(editor: &mut Editor, code: KeyCode) {
    match code {
        KeyCode::Left | KeyCode::Char('h') => editor.move_left(),
        KeyCode::Down | KeyCode::Char('j') => editor.move_down(),
        KeyCode::Up | KeyCode::Char('k') => editor.move_up(),
        KeyCode::Right | KeyCode::Char('l') => editor.move_right(),
        KeyCode::Home | KeyCode::Char('0') => editor.move_to_line_start(),
        KeyCode::End | KeyCode::Char('$') => editor.move_to_line_end(),
        _ => {}
    }
}

fn single_line(value: &str) -> String {
    value
        .chars()
        .filter(|character| !matches!(character, '\r' | '\n'))
        .collect()
}

fn apply_cursor(session: &TerminalSession, cursor: Option<CursorRequest>) -> io::Result<()> {
    let Some(cursor) = cursor else {
        return Ok(());
    };
    let shape = if cursor.style == crossterm::cursor::SetCursorStyle::SteadyBar {
        CursorShape::Bar
    } else {
        CursorShape::Block
    };
    session.set_cursor_shape(shape)
}

struct SessionWriter<'a>(&'a TerminalSession);

impl Write for SessionWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write_ui(bytes)?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
