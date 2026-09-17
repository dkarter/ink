//! Interactive prompt event loop.

use std::io;

use crossterm::event::{Event, KeyEventKind};
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
    terminal::{CursorShape, PromptOutcome, SessionWriter, TerminalSession},
    text::{single_line, textarea},
    ui::{CursorRequest, Input, InputState, Textarea, TextareaState},
};

mod input;

use input::handle_key;

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
        PromptKind::Input => Some(3),
        PromptKind::Textarea if !prompt.fullscreen => Some(6),
        PromptKind::Textarea => None,
    };
    let viewport = match compact_height {
        Some(_) => {
            let (width, height) = crossterm::terminal::size()?;
            let area = compact_area(width, height, compact_height.unwrap());
            session.enter_inline_screen(area.y, area.height)?;
            Viewport::Fixed(area)
        }
        None => {
            session.enter_fullscreen()?;
            Viewport::Fullscreen
        }
    };
    let backend = CrosstermBackend::new(SessionWriter(session));
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport })?;
    let mut input_state = InputState::default();
    let mut textarea_state = TextareaState::default();
    let mut cursor_style = None;
    let mut pending = None;

    loop {
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
                    render_input_status(frame.buffer_mut(), area, editor.mode(), options);
                    cursor = input_state.cursor();
                }
                PromptKind::Textarea => {
                    Textarea::new(&editor)
                        .placeholder(&placeholder)
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
                if let Some(outcome) = handle_key(&mut editor, kind, key, &mut pending) {
                    return Ok(outcome);
                }
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
                session.track_inline_screen(area.y, area.height);
                terminal.resize(area)?;
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}

fn compact_area(width: u16, height: u16, preferred_height: u16) -> Rect {
    let prompt_height = preferred_height.min(height);
    Rect::new(
        0,
        height.saturating_sub(prompt_height),
        width,
        prompt_height,
    )
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
