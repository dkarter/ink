//! Composable Vim-style editor state for input and textarea widgets.

use std::fmt;

use unicode_segmentation::UnicodeSegmentation;

use crate::{display::grapheme_width_at, text};

mod motion;
mod operator;
mod selection;

use motion::{WordClass, word_class};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Motion {
    WordForward,
    WordBackward,
    WordEnd,
    BigWordForward,
    BigWordBackward,
    BigWordEnd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextObject {
    InnerWord,
    AWord,
    InnerBigWord,
    ABigWord,
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
    unnamed_register: Register,
    kind: BufferKind,
    preferred_display_column: Option<usize>,
    insert_origin: Option<Position>,
    insert_advanced: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Register {
    text: String,
    linewise: bool,
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
        let text = text.into();
        Self::with_kind(text::textarea_owned(text), BufferKind::Textarea)
    }

    fn with_kind(text: String, kind: BufferKind) -> Self {
        let mut editor = Self {
            text,
            cursor: Position::default(),
            mode: Mode::Normal,
            anchor: None,
            unnamed_register: Register::default(),
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
        &self.unnamed_register.text
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
        let value = if self.kind == BufferKind::Textarea {
            text::textarea(value)
        } else {
            std::borrow::Cow::Borrowed(value)
        };
        self.text.insert_str(byte, &value);
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

    pub fn move_word_forward(&mut self) {
        self.move_word(true, false);
    }

    pub fn move_word_backward(&mut self) {
        self.move_word(false, false);
    }

    pub fn move_big_word_forward(&mut self) {
        self.move_word(true, true);
    }

    pub fn move_big_word_backward(&mut self) {
        self.move_word(false, true);
    }

    pub fn move_word_end(&mut self) {
        self.move_word_end_with(false);
    }

    pub fn move_big_word_end(&mut self) {
        self.move_word_end_with(true);
    }

    pub fn delete_selection(&mut self) -> bool {
        self.apply_operator(Mode::Normal)
    }

    pub fn change_selection(&mut self) -> bool {
        self.apply_operator(Mode::Insert)
    }

    pub fn delete_motion(&mut self, motion: Motion) -> bool {
        self.apply_motion(motion, Mode::Normal)
    }

    pub fn change_motion(&mut self, motion: Motion) -> bool {
        match motion {
            Motion::WordForward if !self.cursor_is_whitespace(false) => {
                self.change_to_current_word_end(false)
            }
            Motion::BigWordForward if !self.cursor_is_whitespace(true) => {
                self.change_to_current_word_end(true)
            }
            motion => self.apply_motion(motion, Mode::Insert),
        }
    }

    pub fn delete_text_object(&mut self, object: TextObject) -> bool {
        self.text_object_range(object)
            .is_some_and(|range| self.apply_range(range, Mode::Normal))
    }

    pub fn change_text_object(&mut self, object: TextObject) -> bool {
        self.text_object_range(object)
            .is_some_and(|range| self.apply_range(range, Mode::Insert))
    }

    pub fn yank_motion(&mut self, motion: Motion) -> bool {
        let start = self.position_to_byte(self.cursor, false);
        let Some(range) = self.motion_range(motion) else {
            return false;
        };
        self.yank_range(range, start)
    }

    pub fn yank_text_object(&mut self, object: TextObject) -> bool {
        let start = self.position_to_byte(self.cursor, false);
        self.text_object_range(object)
            .is_some_and(|range| self.yank_range(range, start))
    }

    pub fn delete_line(&mut self) -> bool {
        if !self.can_apply_linewise() {
            return false;
        }
        let line = self.cursor.line;
        self.unnamed_register = Register {
            text: self.linewise_text(line, line),
            linewise: true,
        };
        let range = self.line_delete_range(line, line);
        let cursor = range.start;
        self.text.replace_range(range, "");
        self.cursor = self.normal_position_from_byte(cursor.min(self.text.len()));
        self.preferred_display_column = None;
        true
    }

    pub fn change_line(&mut self) -> bool {
        if !self.can_apply_linewise() {
            return false;
        }
        let (start, end) = self.line_bounds(self.cursor.line);
        self.unnamed_register = Register {
            text: self.linewise_text(self.cursor.line, self.cursor.line),
            linewise: true,
        };
        self.text.replace_range(start..end, "");
        self.clear_selection(Mode::Insert);
        self.cursor = self.position_from_byte(start);
        self.insert_origin = Some(self.cursor);
        self.insert_advanced = false;
        self.preferred_display_column = None;
        true
    }

    pub fn yank_line(&mut self) -> bool {
        if !self.can_apply_linewise() {
            return false;
        }
        self.unnamed_register = Register {
            text: self.linewise_text(self.cursor.line, self.cursor.line),
            linewise: true,
        };
        self.preferred_display_column = None;
        true
    }

    pub fn paste_after(&mut self) -> bool {
        self.paste(false)
    }

    pub fn paste_before(&mut self) -> bool {
        self.paste(true)
    }

    pub fn open_line_below(&mut self) -> bool {
        if self.mode != Mode::Normal || self.kind != BufferKind::Textarea {
            return false;
        }
        let insertion = self.line_bounds(self.cursor.line).1;
        self.text.insert(insertion, '\n');
        self.cursor = self.position_from_byte(insertion + 1);
        self.enter_insert();
        true
    }

    pub fn open_line_above(&mut self) -> bool {
        if self.mode != Mode::Normal || self.kind != BufferKind::Textarea {
            return false;
        }
        let insertion = self.line_bounds(self.cursor.line).0;
        self.text.insert(insertion, '\n');
        self.cursor = self.position_from_byte(insertion);
        self.enter_insert();
        true
    }

    pub fn yank_selection(&mut self) -> bool {
        if !self.is_visual() {
            return false;
        }

        let start = self.selection_start();
        let ranges = self.selection_ranges();
        let text = if self.mode == Mode::VisualLine {
            let (first, last) = self.selected_line_interval();
            self.linewise_text(first, last)
        } else {
            self.text_for_ranges(&ranges)
        };
        self.unnamed_register = Register {
            text,
            linewise: self.mode == Mode::VisualLine,
        };
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
        self.unnamed_register = Register {
            text: self.text[start..end].to_owned(),
            linewise: false,
        };
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

    fn display_column(&self, position: Position) -> usize {
        let (start, end) = self.line_bounds(position.line);
        self.text[start..end]
            .graphemes(true)
            .take(position.column)
            .fold(0, |column, grapheme| {
                column + grapheme_width_at(grapheme, column)
            })
    }

    fn position_at_display_column(&self, line: usize, target: usize) -> Position {
        let (start, end) = self.line_bounds(line);
        let mut display = 0;
        let mut column = 0;
        for grapheme in self.text[start..end].graphemes(true) {
            let width = grapheme_width_at(grapheme, display);
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

fn contains_line_break(text: &str) -> bool {
    text.contains(['\n', '\r'])
}
