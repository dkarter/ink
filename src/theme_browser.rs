//! Interactive browser for bundled themes.

use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{
    config::ThemeName,
    editor::Mode,
    terminal::{SessionWriter, TerminalSession},
    theme,
};

pub(crate) enum BrowserOutcome {
    Selected(ThemeName),
    Cancelled,
}

pub(crate) fn run(session: &mut TerminalSession, initial: ThemeName) -> io::Result<BrowserOutcome> {
    session.enter_fullscreen()?;
    let backend = CrosstermBackend::new(SessionWriter(session));
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fullscreen,
        },
    )?;
    let mut selected = ThemeName::ALL
        .iter()
        .position(|theme| *theme == initial)
        .unwrap_or_default();

    let mut dirty = true;
    loop {
        if dirty {
            terminal.draw(|frame| render(frame, selected))?;
            dirty = false;
        }
        match session.read_event()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                let previous = selected;
                if let Some(outcome) = handle_key(key, &mut selected) {
                    return Ok(outcome);
                }
                dirty = selected != previous;
            }
            Event::Resize(_, _) => dirty = true,
            _ => {}
        }
    }
}

fn handle_key(key: KeyEvent, selected: &mut usize) -> Option<BrowserOutcome> {
    match (key.code, key.modifiers) {
        (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) => {
            *selected = selected.saturating_sub(1);
        }
        (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
            *selected = (*selected + 1).min(ThemeName::ALL.len() - 1);
        }
        (KeyCode::Home, _) => *selected = 0,
        (KeyCode::End, _) => *selected = ThemeName::ALL.len() - 1,
        (KeyCode::Enter, _) => {
            return Some(BrowserOutcome::Selected(ThemeName::ALL[*selected]));
        }
        (KeyCode::Esc | KeyCode::Char('q'), KeyModifiers::NONE)
        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            return Some(BrowserOutcome::Cancelled);
        }
        _ => {}
    }
    None
}

fn render(frame: &mut ratatui::Frame<'_>, selected: usize) {
    let selected_theme = ThemeName::ALL[selected];
    let palette = theme::resolve(selected_theme, &Default::default()).palette;
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(palette.background.into())),
        area,
    );

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "Ink themes",
            Style::default()
                .fg(palette.accent.into())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}", selected_theme.as_str()),
            Style::default().fg(palette.foreground.into()),
        ),
    ]));
    frame.render_widget(title, vertical[0]);

    if area.width >= 38 {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(12)])
            .split(vertical[1]);
        render_list(frame, horizontal[0], selected, palette);
        render_preview(frame, horizontal[1], palette);
    } else {
        render_list(frame, vertical[1], selected, palette);
    }

    let controls = if area.width >= 44 {
        "↑/↓ or j/k navigate  enter save  esc/q cancel"
    } else {
        "j/k move  enter save  q quit"
    };
    frame.render_widget(
        Paragraph::new(controls)
            .alignment(Alignment::Center)
            .style(Style::default().fg(palette.muted.into())),
        vertical[2],
    );
}

fn render_list(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    selected: usize,
    palette: theme::Palette,
) {
    let items = ThemeName::ALL
        .iter()
        .map(|theme| ListItem::new(theme.as_str()))
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.border.into()))
                .title(" Bundled palettes "),
        )
        .style(Style::default().fg(palette.foreground.into()))
        .highlight_symbol("› ")
        .highlight_style(
            Style::default()
                .fg(palette.selection_foreground.into())
                .bg(palette.selection.into())
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default().with_selected(Some(selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_preview(frame: &mut ratatui::Frame<'_>, area: Rect, palette: theme::Palette) {
    frame.render_widget(Clear, area);
    let preview = Paragraph::new(vec![
        Line::from(Span::styled(
            "Live preview",
            Style::default()
                .fg(palette.accent.into())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Unicode: café λ 東京",
            Style::default().fg(palette.foreground.into()),
        )),
        Line::from(Span::styled(
            "Secondary guidance",
            Style::default().fg(palette.muted.into()),
        )),
        Line::from(Span::styled(
            " selected text ",
            Style::default()
                .fg(palette.selection_foreground.into())
                .bg(palette.selection.into()),
        )),
        Line::from(Span::styled(
            format!(" {} ", crate::ui::mode_label(Mode::Insert)),
            Style::default()
                .fg(palette.background.into())
                .bg(palette.insert_mode.into()),
        )),
        Line::from(Span::styled(
            format!(" {} ", crate::ui::mode_label(Mode::Normal)),
            Style::default()
                .fg(palette.background.into())
                .bg(palette.normal_mode.into()),
        )),
        Line::from(Span::styled(
            format!(" {} ", crate::ui::mode_label(Mode::Visual)),
            Style::default()
                .fg(palette.background.into())
                .bg(palette.visual_mode.into()),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border.into()))
            .title(" Preview "),
    )
    .style(
        Style::default()
            .fg(palette.foreground.into())
            .bg(palette.background.into()),
    );
    frame.render_widget(preview, area);
}
