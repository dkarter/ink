use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Style,
    widgets::StatefulWidget,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    editor::{Editor, Mode},
    theme::Palette,
};

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
    palette: Option<Palette>,
    show_mode: bool,
}

impl<'a> Input<'a> {
    #[must_use]
    pub const fn new(editor: &'a Editor) -> Self {
        Self {
            editor,
            prompt: "",
            palette: None,
            show_mode: true,
        }
    }

    #[must_use]
    pub const fn prompt(mut self, prompt: &'a str) -> Self {
        self.prompt = prompt;
        self
    }

    #[must_use]
    pub const fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    #[must_use]
    pub const fn show_mode(mut self, show_mode: bool) -> Self {
        self.show_mode = show_mode;
        self
    }
}

impl StatefulWidget for Input<'_> {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.cursor = None;
        if let Some(palette) = self.palette {
            buf.set_style(area, base_style(palette));
        }
        let mode = mode_label(self.editor.mode());
        let prompt_width = text_width(self.prompt);
        let mode_width = text_width(mode);
        let available = usize::from(area.width);
        let (prompt, editor_width, mode_x) = if !self.show_mode && available > prompt_width {
            (Some(self.prompt), available - prompt_width, None)
        } else if !self.show_mode {
            (None, available, None)
        } else if available >= prompt_width + mode_width + 2 {
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
            render_text(
                prompt,
                0,
                area.x,
                area.y,
                prompt_width,
                TextDecoration::plain(self.palette.map(base_style)),
                buf,
            );
        }
        if let Some(x) = mode_x {
            render_text(
                mode,
                0,
                area.x.saturating_add(u16::try_from(x).unwrap_or(u16::MAX)),
                area.y,
                mode_width,
                TextDecoration::plain(
                    self.palette
                        .map(|palette| mode_style(palette, self.editor.mode())),
                ),
                buf,
            );
        }
        render_text(
            self.editor.text(),
            state.viewport.left(),
            editor_x,
            area.y,
            editor_width,
            TextDecoration::selected(
                0,
                &self.editor.selection_ranges(),
                self.palette.map(base_style),
                self.palette.map(selection_style),
            ),
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
    palette: Option<Palette>,
    hint: &'a str,
}

impl<'a> Textarea<'a> {
    #[must_use]
    pub const fn new(editor: &'a Editor) -> Self {
        Self {
            editor,
            palette: None,
            hint: "",
        }
    }

    #[must_use]
    pub const fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    #[must_use]
    pub const fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }
}

impl StatefulWidget for Textarea<'_> {
    type State = TextareaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.cursor = None;
        if let Some(palette) = self.palette {
            buf.set_style(area, base_style(palette));
        }
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

        let ranges = self.editor.selection_ranges();
        let mut byte = 0;
        for (line_index, line) in self.editor.text().split('\n').enumerate() {
            if line_index < state.viewport.top() {
                byte += line.len() + 1;
                continue;
            }
            let row = line_index - state.viewport.top();
            if row >= usize::from(text_height) {
                break;
            }
            render_text(
                line,
                state.viewport.left(),
                area.x,
                area.y
                    .saturating_add(u16::try_from(row).unwrap_or(u16::MAX)),
                text_width,
                TextDecoration::selected(
                    byte,
                    &ranges,
                    self.palette.map(base_style),
                    self.palette.map(selection_style),
                ),
                buf,
            );
            byte += line.len() + 1;
        }

        if area.height > 1 {
            let hint_width = super::display::text_width(self.hint);
            if available > mode_width + hint_width {
                render_text(
                    self.hint,
                    0,
                    area.right()
                        .saturating_sub(u16::try_from(hint_width).unwrap_or(u16::MAX)),
                    area.y.saturating_add(area.height - 1),
                    hint_width,
                    TextDecoration::plain(self.palette.map(muted_style)),
                    buf,
                );
            }
            render_text(
                mode,
                0,
                area.x,
                area.y.saturating_add(area.height - 1),
                usize::from(area.width),
                TextDecoration::plain(
                    self.palette
                        .map(|palette| mode_style(palette, self.editor.mode())),
                ),
                buf,
            );
        } else if let Some(x) = inline_mode_x {
            render_text(
                mode,
                0,
                area.x.saturating_add(u16::try_from(x).unwrap_or(u16::MAX)),
                area.y,
                mode_width,
                TextDecoration::plain(
                    self.palette
                        .map(|palette| mode_style(palette, self.editor.mode())),
                ),
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

fn render_text(
    text: &str,
    offset: usize,
    x: u16,
    y: u16,
    width: usize,
    decoration: TextDecoration<'_>,
    buf: &mut Buffer,
) {
    if width == 0 || !buf.area.contains(Position::new(x, y)) {
        return;
    }
    let original_text = text;
    let (text, leading) = slice_from_cell(original_text, offset);
    if leading >= width {
        return;
    }
    let text = sanitized_prefix(text, width - leading);
    buf.set_stringn(
        x.saturating_add(u16::try_from(leading).unwrap_or(u16::MAX)),
        y,
        text.as_ref(),
        width - leading,
        decoration.render_style.unwrap_or_default(),
    );
    let Some(selection_style) = decoration
        .selection_style
        .filter(|_| !decoration.selection.is_empty())
    else {
        return;
    };
    let mut cell = 0;
    for (byte, grapheme) in original_text.grapheme_indices(true) {
        let grapheme_width = super::display::grapheme_width(grapheme);
        let selected = decoration
            .selection
            .iter()
            .any(|range| range.contains(&(decoration.source_byte + byte)));
        if selected && grapheme_width > 0 && cell + grapheme_width > offset && cell < offset + width
        {
            let left = cell.max(offset) - offset;
            let visible_width = (cell + grapheme_width).min(offset + width) - cell.max(offset);
            buf.set_style(
                Rect::new(
                    x.saturating_add(u16::try_from(left).unwrap_or(u16::MAX)),
                    y,
                    u16::try_from(visible_width).unwrap_or(u16::MAX),
                    1,
                ),
                selection_style,
            );
        }
        cell += grapheme_width;
    }
}

struct TextDecoration<'a> {
    source_byte: usize,
    selection: &'a [Range<usize>],
    render_style: Option<Style>,
    selection_style: Option<Style>,
}

impl<'a> TextDecoration<'a> {
    const fn plain(render_style: Option<Style>) -> Self {
        Self {
            source_byte: 0,
            selection: &[],
            render_style,
            selection_style: None,
        }
    }

    const fn selected(
        source_byte: usize,
        selection: &'a [Range<usize>],
        render_style: Option<Style>,
        selection_style: Option<Style>,
    ) -> Self {
        Self {
            source_byte,
            selection,
            render_style,
            selection_style,
        }
    }
}

fn base_style(palette: Palette) -> Style {
    Style::default()
        .fg(palette.foreground.into())
        .bg(palette.background.into())
}

fn selection_style(palette: Palette) -> Style {
    Style::default()
        .fg(palette.selection_foreground.into())
        .bg(palette.selection.into())
}

fn mode_style(palette: Palette, mode: Mode) -> Style {
    let color = match mode {
        Mode::Insert => palette.insert_mode,
        Mode::Normal => palette.normal_mode,
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => palette.visual_mode,
    };
    Style::default()
        .fg(palette.background.into())
        .bg(color.into())
}

fn muted_style(palette: Palette) -> Style {
    Style::default()
        .fg(palette.muted.into())
        .bg(palette.background.into())
}
