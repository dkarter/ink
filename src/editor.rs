//! Composable Vim-style editor state for input and textarea widgets.

use std::fmt;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod selection;

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
pub enum InputError {
    LineBreak,
}

impl fmt::Display for InputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LineBreak => formatter.write_str("single-line input cannot contain line breaks"),
        }
    }
}

impl std::error::Error for InputError {}

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
    insert_origin: Option<Position>,
    insert_advanced: bool,
}

impl Editor {
    pub fn input(text: impl Into<String>) -> Result<Self, InputError> {
        let text = text.into();
        if contains_line_break(&text) {
            return Err(InputError::LineBreak);
        }
        Ok(Self::with_kind(text, BufferKind::Input))
    }

    #[must_use]
    pub fn empty_input() -> Self {
        Self::with_kind(String::new(), BufferKind::Input)
    }

    #[must_use]
    pub fn textarea(text: impl Into<String>) -> Self {
        Self::with_kind(text.into().replace("\r\n", "\n"), BufferKind::Textarea)
    }

    fn with_kind(text: String, kind: BufferKind) -> Self {
        let mut editor = Self {
            text,
            cursor: Position::default(),
            mode: Mode::Normal,
            anchor: None,
            unnamed_register: String::new(),
            kind,
            preferred_display_column: None,
            insert_origin: None,
            insert_advanced: false,
        };
        editor.cursor = editor.normalized_position(editor.cursor);
        editor
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
        self.cursor = match self.mode {
            Mode::Normal => self.normalized_position(position),
            Mode::Insert => self.clamp_position(position, true),
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => {
                self.clamp_position(position, false)
            }
        };
        self.preferred_display_column = None;
    }

    pub fn enter_insert(&mut self) {
        self.clear_selection(Mode::Insert);
        self.insert_origin = Some(self.cursor);
        self.insert_advanced = false;
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
        let was_insert = self.mode == Mode::Insert;
        if was_insert
            && self.insert_advanced
            && self.cursor.column > 0
            && self
                .insert_origin
                .is_some_and(|origin| origin != self.cursor)
        {
            self.cursor.column -= 1;
        }
        let byte = self.position_to_byte(self.cursor, was_insert);
        self.clear_selection(Mode::Normal);
        self.cursor = self.normal_position_from_byte(byte);
    }

    pub fn insert(&mut self, value: &str) -> bool {
        if self.mode != Mode::Insert
            || (self.kind == BufferKind::Input && contains_line_break(value))
        {
            return false;
        }

        let previous_cursor = self.cursor;
        let byte = self.position_to_byte(previous_cursor, true);
        self.text.insert_str(byte, value);
        self.cursor = self.position_from_byte(byte + value.len());
        self.insert_advanced |= self.cursor != previous_cursor;
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
        let start = self.position_to_byte(start, false);
        self.clear_selection(Mode::Normal);
        self.cursor = self.normal_position_from_byte(start);
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
        self.insert_origin = None;
        self.insert_advanced = false;
    }

    fn clear_selection(&mut self, mode: Mode) {
        self.mode = mode;
        self.anchor = None;
        if mode != Mode::Insert {
            self.insert_origin = None;
            self.insert_advanced = false;
        }
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
        self.insert_origin = (next_mode == Mode::Insert).then_some(self.cursor);
        self.insert_advanced = false;
        self.preferred_display_column = None;
        true
    }

    fn move_vertical(&mut self, direction: isize) {
        if self.kind != BufferKind::Textarea {
            return;
        }
        let display_column = self
            .preferred_display_column
            .unwrap_or_else(|| self.display_column(self.cursor));
        self.preferred_display_column = Some(display_column);
        let last_line = self.line_count().saturating_sub(1);
        let mut line = self
            .cursor
            .line
            .saturating_add_signed(direction)
            .min(last_line);
        if self.mode == Mode::Normal {
            while self.line_grapheme_count(line) == 0 {
                let next = line.saturating_add_signed(direction).min(last_line);
                if next == line {
                    break;
                }
                line = next;
            }
        }
        let position = self.position_at_display_column(line, display_column);
        self.cursor = if self.mode == Mode::Normal && self.line_grapheme_count(line) == 0 {
            self.normalized_position(position)
        } else {
            position
        };
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
        if self.text.is_empty() {
            return Position::default();
        }

        let position = self.position_from_byte(byte);
        if self.line_grapheme_count(position.line) > 0 {
            return self.clamp_position(position, false);
        }
        if let Some((byte, _)) = self.text[..byte.min(self.text.len())]
            .grapheme_indices(true)
            .rev()
            .find(|(_, grapheme)| *grapheme != "\n")
        {
            return self.clamp_position(self.position_from_byte(byte), false);
        }
        let byte = byte.min(self.text.len());
        if let Some((relative, _)) = self.text[byte..]
            .grapheme_indices(true)
            .find(|(_, grapheme)| *grapheme != "\n")
        {
            return self.clamp_position(self.position_from_byte(byte + relative), false);
        }
        Position::default()
    }

    fn normalized_position(&self, position: Position) -> Position {
        self.normal_position_from_byte(self.position_to_byte(position, false))
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

fn contains_line_break(text: &str) -> bool {
    text.contains(['\n', '\r'])
}
