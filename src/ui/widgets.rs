use std::{fmt::Write as _, ops::Range};

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Style,
    widgets::StatefulWidget,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    display::{cursor_width, sanitized_prefix, slice_from_cell, text_width},
    editor::{Editor, Mode},
    theme::Palette,
};

use super::{CursorRequest, Viewport, mode_label};

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
    placeholder: &'a str,
    palette: Option<Palette>,
    show_mode: bool,
    background: bool,
}

impl<'a> Input<'a> {
    #[must_use]
    pub const fn new(editor: &'a Editor) -> Self {
        Self {
            editor,
            prompt: "",
            placeholder: "",
            palette: None,
            show_mode: true,
            background: false,
        }
    }

    #[must_use]
    pub const fn prompt(mut self, prompt: &'a str) -> Self {
        self.prompt = prompt;
        self
    }

    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
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

    #[must_use]
    pub const fn background(mut self, background: bool) -> Self {
        self.background = background;
        self
    }
}

impl StatefulWidget for Input<'_> {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.cursor = None;
        if let Some(palette) = self.palette.filter(|_| self.background) {
            buf.set_style(area, base_style(palette));
        }
        let input_style = self
            .palette
            .map(|palette| input_style(palette, self.background));
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
                TextDecoration::plain(input_style),
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
        let ranges = self.editor.selection_ranges();
        render_text(
            if self.editor.text().is_empty() {
                self.placeholder
            } else {
                self.editor.text()
            },
            state.viewport.left(),
            editor_x,
            area.y,
            editor_width,
            if self.editor.text().is_empty() {
                TextDecoration::plain(
                    self.palette
                        .map(|palette| placeholder_input_style(palette, self.background)),
                )
            } else {
                TextDecoration::selected(0, &ranges, input_style, self.palette.map(selection_style))
            },
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
    placeholder: &'a str,
    palette: Option<Palette>,
    hint: &'a str,
    status_background: bool,
    line_numbers: bool,
}

impl<'a> Textarea<'a> {
    #[must_use]
    pub const fn new(editor: &'a Editor) -> Self {
        Self {
            editor,
            placeholder: "",
            palette: None,
            hint: "",
            status_background: false,
            line_numbers: false,
        }
    }

    #[must_use]
    pub const fn palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    #[must_use]
    pub const fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    #[must_use]
    pub const fn status_background(mut self, status_background: bool) -> Self {
        self.status_background = status_background;
        self
    }

    #[must_use]
    pub const fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
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
        let mode_width = text_width(mode) + 2;
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
        let cursor_cell_width = cursor_width(cursor_line, cursor.column);
        let line_number_digits = if self.line_numbers {
            usize::try_from(line_count.ilog10()).expect("decimal digit count fits usize") + 1
        } else {
            0
        };
        let preferred_gutter_width = line_number_digits + usize::from(self.line_numbers);
        let gutter_width =
            if self.line_numbers && available >= preferred_gutter_width + cursor_cell_width {
                preferred_gutter_width
            } else {
                0
            };
        let content_available = available.saturating_sub(gutter_width);
        let minimum_editor_width = if area.height == 1 {
            cursor_cell_width
        } else {
            1
        };
        let (text_width, inline_mode_x) =
            if area.height == 1 && content_available >= mode_width + minimum_editor_width {
                (content_available - mode_width, Some(available - mode_width))
            } else {
                (content_available, None)
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
        let placeholder = self.editor.text().is_empty();
        let displayed = if placeholder {
            self.placeholder
        } else {
            self.editor.text()
        };
        let line_number_style = self.palette.map(line_number_style).unwrap_or_default();
        let mut line_number = String::with_capacity(gutter_width);
        let mut byte = 0;
        for (line_index, line) in displayed.split('\n').enumerate() {
            if line_index < state.viewport.top() {
                byte += line.len() + 1;
                continue;
            }
            let row = line_index - state.viewport.top();
            if row >= usize::from(text_height) {
                break;
            }
            let y = area
                .y
                .saturating_add(u16::try_from(row).unwrap_or(u16::MAX));
            if gutter_width > 0 && (!placeholder || line_index < line_count) {
                line_number.clear();
                write!(line_number, "{:>line_number_digits$} ", line_index + 1)
                    .expect("writing to a string cannot fail");
                buf.set_stringn(area.x, y, &line_number, gutter_width, line_number_style);
            }
            render_text(
                line,
                state.viewport.left(),
                area.x
                    .saturating_add(u16::try_from(gutter_width).unwrap_or(u16::MAX)),
                y,
                text_width,
                if placeholder {
                    TextDecoration::plain(self.palette.map(placeholder_style))
                } else {
                    TextDecoration::selected(
                        byte,
                        &ranges,
                        self.palette.map(base_style),
                        self.palette.map(selection_style),
                    )
                },
                buf,
            );
            byte += line.len() + 1;
        }

        if area.height > 1 {
            if self.status_background
                && let Some(palette) = self.palette
            {
                buf.set_style(
                    Rect::new(area.x, area.bottom() - 1, area.width, 1),
                    Style::default().bg(palette.status_background.into()),
                );
            }
            let hint_width = crate::display::text_width(self.hint);
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
            if available >= mode_width {
                render_padded_mode(
                    mode,
                    area.x,
                    area.y.saturating_add(area.height - 1),
                    self.palette,
                    self.editor.mode(),
                    buf,
                );
            }
        } else if let Some(x) = inline_mode_x {
            render_padded_mode(
                mode,
                area.x.saturating_add(u16::try_from(x).unwrap_or(u16::MAX)),
                area.y,
                self.palette,
                self.editor.mode(),
                buf,
            );
        }
        state.cursor = Some(CursorRequest::new(
            area.x
                .saturating_add(u16::try_from(gutter_width).unwrap_or(u16::MAX))
                .saturating_add(u16::try_from(visible.column).unwrap_or(u16::MAX)),
            area.y
                .saturating_add(u16::try_from(visible.row).unwrap_or(u16::MAX)),
            self.editor.mode(),
        ));
    }
}

fn render_padded_mode(
    label: &str,
    x: u16,
    y: u16,
    palette: Option<Palette>,
    mode: Mode,
    buf: &mut Buffer,
) {
    let label_width = text_width(label);
    let style = palette.map(|palette| mode_style(palette, mode));
    buf.set_style(
        Rect::new(x, y, u16::try_from(label_width + 2).unwrap_or(u16::MAX), 1),
        style.unwrap_or_default(),
    );
    render_text(
        label,
        0,
        x.saturating_add(1),
        y,
        label_width,
        TextDecoration::plain(style),
        buf,
    );
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
    let text = sanitized_prefix(text, offset + leading, width - leading);
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
        let grapheme_width = crate::display::grapheme_width_at(grapheme, cell);
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

fn input_style(palette: Palette, background: bool) -> Style {
    let style = Style::default().fg(palette.foreground.into());
    if background {
        style.bg(palette.background.into())
    } else {
        style
    }
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

fn line_number_style(palette: Palette) -> Style {
    Style::default()
        .fg(palette.line_number.into())
        .bg(palette.background.into())
}

fn placeholder_style(palette: Palette) -> Style {
    Style::default()
        .fg(palette.placeholder.into())
        .bg(palette.background.into())
}

fn placeholder_input_style(palette: Palette, background: bool) -> Style {
    let style = Style::default().fg(palette.placeholder.into());
    if background {
        style.bg(palette.background.into())
    } else {
        style
    }
}
