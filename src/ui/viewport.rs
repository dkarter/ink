use unicode_segmentation::UnicodeSegmentation;

use crate::display::{boundary_at_or_after, grapheme_width_at};

/// A textarea viewport measured in logical lines and terminal display cells.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Viewport {
    top: usize,
    left: usize,
}

impl Viewport {
    #[must_use]
    pub const fn top(self) -> usize {
        self.top
    }

    #[must_use]
    pub const fn left(self) -> usize {
        self.left
    }

    pub(crate) fn update(
        &mut self,
        line_count: usize,
        cursor_line_text: &str,
        cursor_line: usize,
        cursor_column: usize,
        width: usize,
        height: usize,
    ) -> CursorInViewport {
        let line_count = line_count.max(1);
        let cursor_line = cursor_line.min(line_count - 1);
        if height == 0 {
            self.top = 0;
        } else {
            let max_top = line_count.saturating_sub(height);
            self.top = self.top.min(max_top);
            if cursor_line < self.top {
                self.top = cursor_line;
            } else if cursor_line >= self.top + height {
                self.top = cursor_line + 1 - height;
            }
        }

        let (cursor_start, cursor_width, total_width) =
            line_metrics(cursor_line_text, cursor_column);
        if width == 0 {
            self.left = 0;
        } else {
            self.left = self.left.min(total_width.saturating_sub(width));
            if cursor_start < self.left {
                self.left = cursor_start;
            } else if cursor_start + cursor_width > self.left + width {
                let target = cursor_start + cursor_width - width;
                self.left = boundary_at_or_after(cursor_line_text, target).min(cursor_start);
            }
        }

        CursorInViewport {
            row: cursor_line.saturating_sub(self.top),
            column: cursor_start.saturating_sub(self.left),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CursorInViewport {
    pub row: usize,
    pub column: usize,
}

fn line_metrics(line: &str, column: usize) -> (usize, usize, usize) {
    let mut cursor_start = 0;
    let mut cursor_width = 1;
    let mut total_width = 0;
    for (index, grapheme) in line.graphemes(true).enumerate() {
        let width = grapheme_width_at(grapheme, total_width);
        if index < column {
            cursor_start += width;
        } else if index == column {
            cursor_width = width.max(1);
        }
        total_width += width;
    }
    (cursor_start.min(total_width), cursor_width, total_width)
}
