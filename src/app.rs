//! Interactive prompt event loop.

use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Clear, Paragraph, StatefulWidget, Widget},
};

use crate::{
    cli::{PromptKind, PromptRuntimeOptions, ResolvedPromptOptions},
    config::StartupMode,
    editor::{Editor, Mode},
    terminal::{CursorShape, PromptOutcome, SessionWriter, TerminalSession},
    text::{single_line, textarea},
    ui::{CursorRequest, Input, InputState, Textarea, TextareaState},
};

mod input;

use input::handle_key;

const COMMAND_ERROR_DURATION: Duration = Duration::from_secs(1);

enum CommandLine {
    Editing(Editor),
    Invalid { expires_at: Option<Instant> },
    ConfirmSubmit,
}

enum CommandAction {
    Continue,
    ResumeNormal,
    Outcome(PromptOutcome),
}

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
    let placeholder = match kind {
        PromptKind::Input => std::borrow::Cow::Owned(single_line(&prompt.placeholder)),
        PromptKind::Textarea => textarea(&prompt.placeholder),
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
        PromptKind::Input if !prompt.fullscreen => Some(3),
        PromptKind::Input => None,
        PromptKind::Textarea if !prompt.fullscreen => Some(6),
        PromptKind::Textarea => None,
    };
    let relative = compact_height.is_some();
    let viewport = match compact_height {
        Some(_) => {
            let (width, height) = crossterm::terminal::size()?;
            let area = compact_area(width, height, compact_height.unwrap());
            session.enter_inline_screen(area.height)?;
            Viewport::Fixed(area)
        }
        None => {
            session.enter_fullscreen()?;
            Viewport::Fullscreen
        }
    };
    let backend = CrosstermBackend::new(AnchoredWriter::new(SessionWriter(session), relative));
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;
    let mut input_state = InputState::default();
    let mut command_input_state = InputState::default();
    let mut textarea_state = TextareaState::default();
    let mut cursor_style = None;
    let mut pending = None;
    let mut command_line = None;

    loop {
        if matches!(
            command_line.as_ref(),
            Some(CommandLine::Invalid { expires_at: Some(expires_at) })
                if Instant::now() >= *expires_at
        ) {
            command_line = None;
        }
        let mut cursor = None;
        terminal.draw(|frame| {
            let area = frame.area();
            match kind {
                PromptKind::Input => {
                    let input_area = Rect::new(area.x, area.y, area.width, area.height.min(1));
                    Input::new(&editor)
                        .prompt(&prompt.prompt)
                        .placeholder(&placeholder)
                        .palette(options.theme.palette)
                        .background(options.input_background)
                        .show_mode(false)
                        .render(input_area, frame.buffer_mut(), &mut input_state);
                    cursor = if let Some(command) = command_line.as_ref() {
                        render_command_line(
                            frame.buffer_mut(),
                            area,
                            command,
                            &mut command_input_state,
                            options,
                        )
                    } else {
                        render_input_status(frame.buffer_mut(), area, editor.mode(), options);
                        input_state.cursor()
                    };
                }
                PromptKind::Textarea => {
                    Textarea::new(&editor)
                        .placeholder(&placeholder)
                        .palette(options.theme.palette)
                        .hint("ctrl-d submit  ctrl-c cancel")
                        .status_background(options.status_background)
                        .render(area, frame.buffer_mut(), &mut textarea_state);
                    cursor = command_line.as_ref().map_or_else(
                        || textarea_state.cursor(),
                        |command| {
                            render_command_line(
                                frame.buffer_mut(),
                                area,
                                command,
                                &mut command_input_state,
                                options,
                            )
                        },
                    );
                }
            }
            if let Some(request) = cursor {
                frame.set_cursor_position(request.position);
            }
        })?;
        if let Some(CommandLine::Invalid { expires_at }) = command_line.as_mut()
            && expires_at.is_none()
        {
            *expires_at = Some(Instant::now() + COMMAND_ERROR_DURATION);
        }
        if cursor.map(|request| request.style) != cursor_style {
            apply_cursor(session, cursor)?;
            session.flush()?;
            cursor_style = cursor.map(|request| request.style);
        }

        let event = if let Some(CommandLine::Invalid {
            expires_at: Some(expires_at),
        }) = command_line.as_ref()
        {
            let Some(event) =
                session.poll_event(expires_at.saturating_duration_since(Instant::now()))?
            else {
                command_line = None;
                continue;
            };
            event
        } else {
            session.read_event()?
        };
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if command_line.is_some() {
                    match handle_command_key(&mut command_line, key, editor.text()) {
                        CommandAction::Continue => continue,
                        CommandAction::ResumeNormal => {}
                        CommandAction::Outcome(outcome) => return Ok(outcome),
                    }
                }
                if editor.mode() == Mode::Normal
                    && pending.is_none()
                    && key.code == KeyCode::Char(':')
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                {
                    let mut command = Editor::empty_input();
                    command.enter_insert();
                    command_line = Some(CommandLine::Editing(command));
                    continue;
                }
                if let Some(outcome) = handle_key(&mut editor, kind, key, &mut pending) {
                    return Ok(outcome);
                }
            }
            Event::Paste(value)
                if matches!(command_line.as_ref(), Some(CommandLine::Editing(_))) =>
            {
                let Some(CommandLine::Editing(command)) = command_line.as_mut() else {
                    unreachable!("command line state was checked");
                };
                command.insert(
                    &value
                        .chars()
                        .filter(|character| !character.is_control())
                        .collect::<String>(),
                );
            }
            Event::Paste(value) if editor.mode() == Mode::Insert => {
                let value = if kind == PromptKind::Input {
                    single_line(&value)
                } else {
                    value
                };
                editor.insert(&value);
            }
            Event::Resize(width, height) if compact_height.is_some() => {
                let area = compact_area(width, height, compact_height.unwrap());
                session.track_inline_screen(area.height);
                terminal.resize(area)?;
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}

fn compact_area(width: u16, height: u16, preferred_height: u16) -> Rect {
    let prompt_height = preferred_height.min(height);
    Rect::new(0, 0, width, prompt_height)
}

// Ratatui emits absolute cursor positions. Compact prompts translate those positions relative to
// the terminal's live cursor so they can start at the command invocation without querying its row.
struct AnchoredWriter<'a> {
    inner: SessionWriter<'a>,
    relative: bool,
    pending: Vec<u8>,
    translated: Vec<u8>,
    row: u16,
}

impl<'a> AnchoredWriter<'a> {
    fn new(inner: SessionWriter<'a>, relative: bool) -> Self {
        Self {
            inner,
            relative,
            pending: Vec::new(),
            translated: Vec::new(),
            row: 0,
        }
    }
}

impl Write for AnchoredWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.relative {
            self.pending.extend_from_slice(bytes);
            Ok(bytes.len())
        } else {
            self.inner.write(bytes)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.relative && !self.pending.is_empty() {
            let previous_row = self.row;
            self.translated.clear();
            relative_cursor_moves(&self.pending, &mut self.row, &mut self.translated);
            self.pending.clear();
            if let Err(error) = self.inner.write_all(&self.translated) {
                self.inner.0.abandon_inline_screen();
                return Err(error);
            }
            if self.row != previous_row {
                self.inner.0.track_inline_cursor(self.row);
            }
        }
        self.inner.flush()
    }
}

fn relative_cursor_moves(bytes: &[u8], current_row: &mut u16, translated: &mut Vec<u8>) {
    translated.reserve(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let Some((consumed, x, y)) = parse_cursor_position(&bytes[index..]) else {
            translated.push(bytes[index]);
            index += 1;
            continue;
        };
        translated.push(b'\r');
        if y < *current_row {
            write!(translated, "\x1b[{}A", *current_row - y)
                .expect("writing to a vector cannot fail");
        } else if y > *current_row {
            write!(translated, "\x1b[{}B", y - *current_row)
                .expect("writing to a vector cannot fail");
        }
        if x > 0 {
            write!(translated, "\x1b[{x}C").expect("writing to a vector cannot fail");
        }
        *current_row = y;
        index += consumed;
    }
}

fn parse_cursor_position(bytes: &[u8]) -> Option<(usize, u16, u16)> {
    if !bytes.starts_with(b"\x1b[") {
        return None;
    }
    let mut index = 2;
    let row = parse_decimal(bytes, &mut index)?;
    if bytes.get(index) != Some(&b';') {
        return None;
    }
    index += 1;
    let column = parse_decimal(bytes, &mut index)?;
    if bytes.get(index) != Some(&b'H') {
        return None;
    }
    Some((index + 1, column.saturating_sub(1), row.saturating_sub(1)))
}

fn parse_decimal(bytes: &[u8], index: &mut usize) -> Option<u16> {
    let start = *index;
    let mut value = 0_u16;
    while let Some(digit) = bytes.get(*index).and_then(|byte| byte.checked_sub(b'0')) {
        if digit > 9 {
            break;
        }
        value = value.checked_mul(10)?.checked_add(u16::from(digit))?;
        *index += 1;
    }
    (*index > start).then_some(value)
}

fn render_input_status(
    buffer: &mut ratatui::buffer::Buffer,
    area: Rect,
    mode: Mode,
    options: ResolvedPromptOptions,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let palette = options.theme.palette;
    let status_area = Rect::new(area.x, area.bottom() - 1, area.width, 1);
    if options.status_background {
        buffer.set_style(
            status_area,
            Style::default().bg(palette.status_background.into()),
        );
    }
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

fn render_command_line(
    buffer: &mut ratatui::buffer::Buffer,
    area: Rect,
    command: &CommandLine,
    state: &mut InputState,
    options: ResolvedPromptOptions,
) -> Option<CursorRequest> {
    if area.height < 2 || area.width == 0 {
        return None;
    }
    let status_area = Rect::new(area.x, area.bottom() - 1, area.width, 1);
    Clear.render(status_area, buffer);
    let mut palette = options.theme.palette;
    palette.background = palette.status_background;
    match command {
        CommandLine::Editing(command) => {
            Input::new(command)
                .prompt(":")
                .palette(palette)
                .background(options.status_background)
                .show_mode(false)
                .render(status_area, buffer, state);
            state.cursor()
        }
        CommandLine::Invalid { .. } => {
            render_status_message(
                buffer,
                status_area,
                "not a valid command",
                palette.error,
                options.status_background,
                palette.status_background,
            );
            None
        }
        CommandLine::ConfirmSubmit => {
            render_status_message(
                buffer,
                status_area,
                "Submit? (y/n)",
                palette.foreground,
                options.status_background,
                palette.status_background,
            );
            None
        }
    }
}

fn render_status_message(
    buffer: &mut ratatui::buffer::Buffer,
    area: Rect,
    message: &str,
    foreground: crate::theme::Color,
    filled: bool,
    background: crate::theme::Color,
) {
    let style = Style::default().fg(foreground.into());
    Paragraph::new(message)
        .style(if filled {
            style.bg(background.into())
        } else {
            style
        })
        .render(area, buffer);
}

fn handle_command_key(
    command_line: &mut Option<CommandLine>,
    key: KeyEvent,
    value: &str,
) -> CommandAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return CommandAction::Outcome(PromptOutcome::Cancelled(
            crate::terminal::CancelReason::Interrupt,
        ));
    }
    match command_line.as_mut().expect("command line is active") {
        CommandLine::Invalid { .. } => {
            *command_line = None;
            return CommandAction::ResumeNormal;
        }
        CommandLine::ConfirmSubmit => {
            let unmodified = !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
            return match key.code {
                KeyCode::Char('y' | 'Y') if unmodified => {
                    CommandAction::Outcome(PromptOutcome::Accepted(value.to_owned()))
                }
                KeyCode::Char('n' | 'N') if unmodified => {
                    *command_line = None;
                    CommandAction::Continue
                }
                KeyCode::Esc => {
                    *command_line = None;
                    CommandAction::Continue
                }
                _ => CommandAction::Continue,
            };
        }
        CommandLine::Editing(_) => {}
    }
    match key.code {
        KeyCode::Esc => *command_line = None,
        KeyCode::Backspace => {
            let Some(CommandLine::Editing(command)) = command_line.as_mut() else {
                unreachable!("command line state was checked");
            };
            command.backspace();
        }
        KeyCode::Enter => match command_line.as_ref() {
            Some(CommandLine::Editing(command)) if command.text() == "wq" => {
                return CommandAction::Outcome(PromptOutcome::Accepted(value.to_owned()));
            }
            Some(CommandLine::Editing(command)) if command.text() == "q!" => {
                return CommandAction::Outcome(PromptOutcome::Cancelled(
                    crate::terminal::CancelReason::NormalQuit,
                ));
            }
            Some(CommandLine::Editing(command)) if matches!(command.text(), "q" | "qa") => {
                *command_line = Some(CommandLine::ConfirmSubmit);
            }
            Some(CommandLine::Editing(_)) => {
                *command_line = Some(CommandLine::Invalid { expires_at: None });
            }
            Some(CommandLine::Invalid { .. } | CommandLine::ConfirmSubmit) | None => {
                unreachable!("command line state was checked");
            }
        },
        KeyCode::Char(character)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            let Some(CommandLine::Editing(command)) = command_line.as_mut() else {
                unreachable!("command line state was checked");
            };
            command.insert(&character.to_string());
        }
        _ => {}
    }
    CommandAction::Continue
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
