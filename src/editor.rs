//! Composable Vim-style editor state for input and textarea widgets.

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    #[must_use]
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    VisualLine,
    VisualBlock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BufferKind {
    Input,
    Textarea,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Editor {
    text: String,
    cursor: Position,
    mode: Mode,
    anchor: Option<Position>,
    unnamed_register: String,
    kind: BufferKind,
    preferred_display_column: Option<usize>,
}

impl Editor {
    #[must_use]
    pub fn input(text: impl Into<String>) -> Self {
        let text = text.into();
        assert!(
            !text.contains(['\n', '\r']),
            "single-line editor text cannot contain line breaks"
        );
        Self::with_kind(text, BufferKind::Input)
    }

    #[must_use]
    pub fn textarea(text: impl Into<String>) -> Self {
        Self::with_kind(text.into().replace("\r\n", "\n"), BufferKind::Textarea)
    }

    fn with_kind(text: String, kind: BufferKind) -> Self {
        Self {
            text,
            cursor: Position::default(),
            mode: Mode::Normal,
            anchor: None,
            unnamed_register: String::new(),
            kind,
            preferred_display_column: None,
        }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub const fn cursor(&self) -> Position {
        self.cursor
    }

    #[must_use]
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    #[must_use]
    pub fn unnamed_register(&self) -> &str {
        &self.unnamed_register
    }

    pub fn set_cursor(&mut self, position: Position) {
        self.cursor = self.clamp_position(position, self.mode == Mode::Insert);
        self.preferred_display_column = None;
    }

    pub fn enter_insert(&mut self) {
        self.clear_selection(Mode::Insert);
    }

    pub fn enter_visual(&mut self) {
        self.start_selection(Mode::Visual);
    }

    pub fn enter_visual_line(&mut self) {
        if self.kind == BufferKind::Textarea {
            self.start_selection(Mode::VisualLine);
        }
    }

    pub fn enter_visual_block(&mut self) {
        if self.kind == BufferKind::Textarea {
            self.start_selection(Mode::VisualBlock);
        }
    }

    pub fn escape(&mut self) {
        if self.mode == Mode::Insert && self.cursor.column > 0 {
            self.cursor.column -= 1;
        }
        self.clear_selection(Mode::Normal);
        self.cursor = self.clamp_position(self.cursor, false);
    }

    pub fn insert(&mut self, value: &str) -> bool {
        if self.mode != Mode::Insert
            || (self.kind == BufferKind::Input && value.contains(['\n', '\r']))
        {
            return false;
        }

        let byte = self.position_to_byte(self.cursor, true);
        self.text.insert_str(byte, value);
        self.cursor = self.position_from_byte(byte + value.len());
        self.preferred_display_column = None;
        true
    }

    pub fn move_left(&mut self) {
        self.cursor.column = self.cursor.column.saturating_sub(1);
        self.preferred_display_column = None;
    }

    pub fn move_right(&mut self) {
        let length = self.line_grapheme_count(self.cursor.line);
        let maximum = if self.mode == Mode::Insert {
            length
        } else {
            length.saturating_sub(1)
        };
        self.cursor.column = self.cursor.column.saturating_add(1).min(maximum);
        self.preferred_display_column = None;
    }

    pub fn move_up(&mut self) {
        self.move_vertical(-1);
    }

    pub fn move_down(&mut self) {
        self.move_vertical(1);
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor.column = 0;
        self.preferred_display_column = None;
    }

    pub fn move_to_line_end(&mut self) {
        let length = self.line_grapheme_count(self.cursor.line);
        self.cursor.column = if self.mode == Mode::Insert {
            length
        } else {
            length.saturating_sub(1)
        };
        self.preferred_display_column = None;
    }

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

    pub fn delete_selection(&mut self) -> bool {
        self.apply_operator(Mode::Normal)
    }

    pub fn change_selection(&mut self) -> bool {
        self.apply_operator(Mode::Insert)
    }

    pub fn yank_selection(&mut self) -> bool {
        if !self.is_visual() {
            return false;
        }

        let start = self.selection_start();
        let ranges = self.selection_ranges();
        self.unnamed_register = self.text_for_ranges(&ranges);
        self.clear_selection(Mode::Normal);
        self.cursor = self.clamp_position(start, false);
        true
    }

    pub fn delete_at_cursor(&mut self) -> bool {
        if self.mode != Mode::Normal {
            return false;
        }
        let start = self.position_to_byte(self.cursor, false);
        let end = self.next_grapheme_boundary(start);
        if start == end || self.text[start..end].contains('\n') {
            return false;
        }
        self.unnamed_register = self.text[start..end].to_owned();
        self.text.replace_range(start..end, "");
        self.cursor = self.normal_position_from_byte(start);
        self.preferred_display_column = None;
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.mode != Mode::Insert || self.cursor == Position::default() {
            return false;
        }
        let end = self.position_to_byte(self.cursor, true);
        let start = self.previous_grapheme_boundary(end);
        self.text.replace_range(start..end, "");
        self.cursor = self.position_from_byte(start);
        self.preferred_display_column = None;
        true
    }

    fn start_selection(&mut self, mode: Mode) {
        self.cursor = self.clamp_position(self.cursor, false);
        self.mode = mode;
        self.anchor = Some(self.cursor);
    }

    fn clear_selection(&mut self, mode: Mode) {
        self.mode = mode;
        self.anchor = None;
    }

    fn is_visual(&self) -> bool {
        matches!(
            self.mode,
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock
        )
    }

    fn apply_operator(&mut self, next_mode: Mode) -> bool {
        if !self.is_visual() {
            return false;
        }

        let ranges = self.selection_ranges();
        let start = ranges.first().map_or(0, |range| range.start);
        self.unnamed_register = self.text_for_ranges(&ranges);
        for range in ranges.into_iter().rev() {
            self.text.replace_range(range, "");
        }
        self.clear_selection(next_mode);
        self.cursor = if next_mode == Mode::Insert {
            self.position_from_byte(start.min(self.text.len()))
        } else {
            self.normal_position_from_byte(start.min(self.text.len()))
        };
        self.preferred_display_column = None;
        true
    }

    fn text_for_ranges(&self, ranges: &[Range<usize>]) -> String {
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

    fn selection_start(&self) -> Position {
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

    fn move_vertical(&mut self, direction: isize) {
        if self.kind != BufferKind::Textarea {
            return;
        }
        let display_column = self
            .preferred_display_column
            .unwrap_or_else(|| self.display_column(self.cursor));
        self.preferred_display_column = Some(display_column);
        let line = self
            .cursor
            .line
            .saturating_add_signed(direction)
            .min(self.line_count().saturating_sub(1));
        self.cursor = self.position_at_display_column(line, display_column);
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

    fn line_range(&self, first: usize, second: usize) -> Range<usize> {
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
            let width = display_width(grapheme);
            let end_column = display + width - 1;
            if display <= right && end_column >= left {
                selected_start.get_or_insert(line_start + relative);
                selected_end = Some(line_start + relative + grapheme.len());
            }
            display += width;
        }
        selected_start.unwrap_or(line_end)..selected_end.unwrap_or(line_end)
    }

    fn display_column(&self, position: Position) -> usize {
        let (start, end) = self.line_bounds(position.line);
        self.text[start..end]
            .graphemes(true)
            .take(position.column)
            .map(display_width)
            .sum()
    }

    fn position_at_display_column(&self, line: usize, target: usize) -> Position {
        let (start, end) = self.line_bounds(line);
        let mut display = 0;
        let mut column = 0;
        for grapheme in self.text[start..end].graphemes(true) {
            let width = display_width(grapheme);
            if display + width > target {
                return Position::new(line, column);
            }
            display += width;
            column += 1;
        }
        if self.mode == Mode::Insert {
            Position::new(line, column)
        } else {
            Position::new(line, column.saturating_sub(1))
        }
    }

    fn clamp_position(&self, position: Position, allow_end: bool) -> Position {
        let line = position.line.min(self.line_count().saturating_sub(1));
        let count = self.line_grapheme_count(line);
        let maximum = if allow_end {
            count
        } else {
            count.saturating_sub(1)
        };
        Position::new(line, position.column.min(maximum))
    }

    fn position_to_byte(&self, position: Position, allow_end: bool) -> usize {
        let position = self.clamp_position(position, allow_end);
        let (start, end) = self.line_bounds(position.line);
        self.text[start..end]
            .grapheme_indices(true)
            .nth(position.column)
            .map_or(end, |(relative, _)| start + relative)
    }

    fn position_from_byte(&self, byte: usize) -> Position {
        let byte = byte.min(self.text.len());
        let mut line = 0;
        let mut line_start = 0;
        for (index, character) in self.text.char_indices() {
            if index >= byte {
                break;
            }
            if character == '\n' {
                line += 1;
                line_start = index + 1;
            }
        }
        let column = self.text[line_start..byte]
            .graphemes(true)
            .filter(|grapheme| *grapheme != "\n")
            .count();
        Position::new(line, column)
    }

    fn normal_position_from_byte(&self, byte: usize) -> Position {
        self.clamp_position(self.position_from_byte(byte), false)
    }

    fn line_count(&self) -> usize {
        self.text.bytes().filter(|byte| *byte == b'\n').count() + 1
    }

    fn line_bounds(&self, target: usize) -> (usize, usize) {
        let mut line = 0;
        let mut start = 0;
        for (index, byte) in self.text.bytes().enumerate() {
            if byte == b'\n' {
                if line == target {
                    return (start, index);
                }
                line += 1;
                start = index + 1;
            }
        }
        (start, self.text.len())
    }

    fn line_grapheme_count(&self, line: usize) -> usize {
        let (start, end) = self.line_bounds(line);
        self.text[start..end].graphemes(true).count()
    }

    fn next_grapheme_boundary(&self, byte: usize) -> usize {
        self.text[byte..]
            .grapheme_indices(true)
            .nth(1)
            .map_or(self.text.len(), |(relative, _)| byte + relative)
    }

    fn previous_grapheme_boundary(&self, byte: usize) -> usize {
        self.text[..byte]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(index, _)| index)
    }
}

fn display_width(grapheme: &str) -> usize {
    UnicodeWidthStr::width(grapheme).max(1)
}
