use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::display::grapheme_width_at;

use super::{Editor, Mode, Position};

impl Editor {
    #[must_use]
    pub fn selection_ranges(&self) -> Vec<Range<usize>> {
        let Some(anchor) = self.anchor else {
            return Vec::new();
        };

        match self.mode {
            Mode::Visual => vec![self.character_range(anchor, self.cursor)],
            Mode::VisualLine => vec![self.line_range(anchor.line, self.cursor.line)],
            Mode::VisualBlock => self.block_ranges(anchor, self.cursor),
            Mode::Normal | Mode::Insert => Vec::new(),
        }
    }

    #[must_use]
    pub fn selected_text(&self) -> String {
        self.text_for_ranges(&self.selection_ranges())
    }

    pub(super) fn text_for_ranges(&self, ranges: &[Range<usize>]) -> String {
        let separator = if self.mode == Mode::VisualBlock {
            "\n"
        } else {
            ""
        };
        let capacity = ranges.iter().map(Range::len).sum::<usize>()
            + separator.len() * ranges.len().saturating_sub(1);
        let mut selected = String::with_capacity(capacity);
        for (index, range) in ranges.iter().enumerate() {
            if index > 0 {
                selected.push_str(separator);
            }
            selected.push_str(&self.text[range.clone()]);
        }
        selected
    }

    pub(super) fn selection_start(&self) -> Position {
        let anchor = self.anchor.unwrap_or(self.cursor);
        match self.mode {
            Mode::Visual => anchor.min(self.cursor),
            Mode::VisualLine => Position::new(anchor.line.min(self.cursor.line), 0),
            Mode::VisualBlock => {
                let top = anchor.line.min(self.cursor.line);
                let left = self
                    .display_column(anchor)
                    .min(self.display_column(self.cursor));
                self.position_at_display_column(top, left)
            }
            Mode::Normal | Mode::Insert => self.cursor,
        }
    }

    fn character_range(&self, first: Position, second: Position) -> Range<usize> {
        let (start, end) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        let start = self.position_to_byte(start, false);
        let end = self.next_grapheme_boundary(self.position_to_byte(end, false));
        start..end
    }

    pub(super) fn line_range(&self, first: usize, second: usize) -> Range<usize> {
        let start_line = first.min(second);
        let end_line = first.max(second);
        let start = self.line_bounds(start_line).0;
        let (_, content_end) = self.line_bounds(end_line);
        let end = if content_end < self.text.len() {
            content_end + 1
        } else {
            content_end
        };
        start..end
    }

    fn block_ranges(&self, first: Position, second: Position) -> Vec<Range<usize>> {
        let top = first.line.min(second.line);
        let bottom = first.line.max(second.line);
        let first_column = self.display_column(first);
        let second_column = self.display_column(second);
        let left = first_column.min(second_column);
        let right = first_column.max(second_column);

        let mut ranges = Vec::with_capacity(bottom - top + 1);
        let mut line_start = self.line_bounds(top).0;
        for _ in top..=bottom {
            let line_end = self.text[line_start..]
                .find('\n')
                .map_or(self.text.len(), |relative| line_start + relative);
            ranges.push(self.display_range(line_start, line_end, left, right));
            line_start = (line_end + 1).min(self.text.len());
        }
        ranges
    }

    fn display_range(
        &self,
        line_start: usize,
        line_end: usize,
        left: usize,
        right: usize,
    ) -> Range<usize> {
        let mut display = 0;
        let mut selected_start = None;
        let mut selected_end = None;
        for (relative, grapheme) in self.text[line_start..line_end].grapheme_indices(true) {
            let width = grapheme_width_at(grapheme, display);
            if width == 0 {
                if selected_start.is_some() {
                    selected_end = Some(line_start + relative + grapheme.len());
                }
                continue;
            }
            let end_column = display + width - 1;
            if display <= right && end_column >= left {
                selected_start.get_or_insert(line_start + relative);
                selected_end = Some(line_start + relative + grapheme.len());
            }
            display += width;
        }
        selected_start.unwrap_or(line_end)..selected_end.unwrap_or(line_end)
    }
}
