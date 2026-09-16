use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Style,
    widgets::StatefulWidget,
};

use crate::editor::Editor;

use super::{
    CursorRequest, Viewport,
    display::{cursor_width, sanitized_prefix, slice_from_cell, text_width},
    mode_label,
};

/// Rendering state retained by a single-line input.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InputState {
    viewport: Viewport,
    cursor: Option<CursorRequest>,
}

impl InputState {
    #[must_use]
    pub const fn horizontal_offset(&self) -> usize {
        self.viewport.left()
    }

    #[must_use]
    pub const fn cursor(&self) -> Option<CursorRequest> {
        self.cursor
    }
}

/// A responsive single-line editor widget.
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    editor: &'a Editor,
    prompt: &'a str,
}

impl<'a> Input<'a> {
    #[must_use]
    pub const fn new(editor: &'a Editor) -> Self {
        Self { editor, prompt: "" }
    }

    #[must_use]
    pub const fn prompt(mut self, prompt: &'a str) -> Self {
        self.prompt = prompt;
        self
    }
}

impl StatefulWidget for Input<'_> {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.cursor = None;
        let mode = mode_label(self.editor.mode());
        let prompt_width = text_width(self.prompt);
        let mode_width = text_width(mode);
        let available = usize::from(area.width);
        let (prompt, editor_width, mode_x) = if available >= prompt_width + mode_width + 2 {
            (
                Some(self.prompt),
                available - prompt_width - mode_width - 1,
                Some(available - mode_width),
            )
        } else if available >= mode_width + 2 {
            (
                None,
                available - mode_width - 1,
                Some(available - mode_width),
            )
        } else {
            (None, available, None)
        };

        let editor_x = prompt.map_or(area.x, |_| {
            area.x
                .saturating_add(u16::try_from(prompt_width).unwrap_or(u16::MAX))
        });
        let cursor = self.editor.cursor();
        let visible =
            state
                .viewport
                .update(1, self.editor.text(), 0, cursor.column, editor_width, 1);
        if area.width == 0 || area.height == 0 {
            return;
        }
        if let Some(prompt) = prompt {
            render_text(prompt, 0, area.x, area.y, prompt_width, buf);
        }
        if let Some(x) = mode_x {
            render_text(
                mode,
                0,
                area.x.saturating_add(u16::try_from(x).unwrap_or(u16::MAX)),
                area.y,
                mode_width,
                buf,
            );
        }
        render_text(
            self.editor.text(),
            state.viewport.left(),
            editor_x,
            area.y,
            editor_width,
            buf,
        );
        state.cursor = Some(CursorRequest::new(
            editor_x.saturating_add(u16::try_from(visible.column).unwrap_or(u16::MAX)),
            area.y,
            self.editor.mode(),
        ));
    }
}

/// Rendering state retained by a multiline textarea.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextareaState {
    viewport: Viewport,
    cursor: Option<CursorRequest>,
}

impl TextareaState {
    #[must_use]
    pub const fn viewport(&self) -> Viewport {
        self.viewport
    }

    #[must_use]
    pub const fn cursor(&self) -> Option<CursorRequest> {
        self.cursor
    }
}

/// A responsive multiline editor widget.
#[derive(Clone, Copy, Debug)]
pub struct Textarea<'a> {
    editor: &'a Editor,
}

impl<'a> Textarea<'a> {
    #[must_use]
    pub const fn new(editor: &'a Editor) -> Self {
        Self { editor }
    }
}

impl StatefulWidget for Textarea<'_> {
    type State = TextareaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.cursor = None;
        let text_height = if area.height > 1 {
            area.height - 1
        } else {
            area.height
        };
        let mode = mode_label(self.editor.mode());
        let mode_width = text_width(mode);
        let available = usize::from(area.width);
        let cursor = self.editor.cursor();
        let line_count = self
            .editor
            .text()
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let cursor_line = self
            .editor
            .text()
            .split('\n')
            .nth(cursor.line)
            .unwrap_or_default();
        let minimum_editor_width = if area.height == 1 {
            cursor_width(cursor_line, cursor.column)
        } else {
            1
        };
        let (text_width, inline_mode_x) =
            if area.height == 1 && available > mode_width + minimum_editor_width {
                (available - mode_width - 1, Some(available - mode_width))
            } else {
                (available, None)
            };
        let visible = state.viewport.update(
            line_count,
            cursor_line,
            cursor.line,
            cursor.column,
            text_width,
            usize::from(text_height),
        );
        if area.width == 0 || area.height == 0 {
            return;
        }

        for (row, line) in self
            .editor
            .text()
            .split('\n')
            .skip(state.viewport.top())
            .take(usize::from(text_height))
            .enumerate()
        {
            render_text(
                line,
                state.viewport.left(),
                area.x,
                area.y
                    .saturating_add(u16::try_from(row).unwrap_or(u16::MAX)),
                text_width,
                buf,
            );
        }

        if area.height > 1 {
            render_text(
                mode,
                0,
                area.x,
                area.y.saturating_add(area.height - 1),
                usize::from(area.width),
                buf,
            );
        } else if let Some(x) = inline_mode_x {
            render_text(
                mode,
                0,
                area.x.saturating_add(u16::try_from(x).unwrap_or(u16::MAX)),
                area.y,
                mode_width,
                buf,
            );
        }
        state.cursor = Some(CursorRequest::new(
            area.x
                .saturating_add(u16::try_from(visible.column).unwrap_or(u16::MAX)),
            area.y
                .saturating_add(u16::try_from(visible.row).unwrap_or(u16::MAX)),
            self.editor.mode(),
        ));
    }
}

fn render_text(text: &str, offset: usize, x: u16, y: u16, width: usize, buf: &mut Buffer) {
    if width == 0 || !buf.area.contains(Position::new(x, y)) {
        return;
    }
    let (text, leading) = slice_from_cell(text, offset);
    if leading >= width {
        return;
    }
    let text = sanitized_prefix(text, width - leading);
    buf.set_stringn(
        x.saturating_add(u16::try_from(leading).unwrap_or(u16::MAX)),
        y,
        text.as_ref(),
        width - leading,
        Style::default(),
    );
}
