use crossterm::cursor::SetCursorStyle;
use ratatui::layout::Position;

use crate::editor::Mode;

/// The cursor placement and shape to apply after drawing a prompt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CursorRequest {
    pub position: Position,
    pub style: SetCursorStyle,
}

impl CursorRequest {
    pub(crate) const fn new(x: u16, y: u16, mode: Mode) -> Self {
        Self {
            position: Position::new(x, y),
            style: cursor_style(mode),
        }
    }
}

/// Returns the steady cursor shape associated with an editing mode.
#[must_use]
pub const fn cursor_style(mode: Mode) -> SetCursorStyle {
    match mode {
        Mode::Insert => SetCursorStyle::SteadyBar,
        Mode::Normal | Mode::Visual | Mode::VisualLine | Mode::VisualBlock => {
            SetCursorStyle::SteadyBlock
        }
    }
}

/// Returns the visible, color-independent label for an editing mode.
#[must_use]
pub const fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Insert => "INSERT",
        Mode::Normal => "NORMAL",
        Mode::Visual => "VISUAL",
        Mode::VisualLine => "VISUAL LINE",
        Mode::VisualBlock => "VISUAL BLOCK",
    }
}
