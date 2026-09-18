use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use super::{BufferKind, Editor, Mode, Motion, Register, TextObject, WordClass, word_class};

impl Editor {
    pub(super) fn apply_operator(&mut self, next_mode: Mode) -> bool {
        if !self.is_visual() {
            return false;
        }
        self.begin_change();
        if self.mode == Mode::VisualLine {
            return self.apply_visual_line_operator(next_mode);
        }
        let ranges = self.selection_ranges();
        let block_offsets =
            (self.mode == Mode::VisualBlock && next_mode == Mode::Insert).then(|| {
                let mut removed = 0;
                ranges
                    .iter()
                    .map(|range| {
                        let offset = range.start - removed;
                        removed += range.len();
                        offset
                    })
                    .collect::<Vec<_>>()
            });
        let start = ranges.first().map_or(0, |range| range.start);
        self.unnamed_register = Register {
            text: self.text_for_ranges(&ranges),
            linewise: false,
        };
        for range in ranges.into_iter().rev() {
            self.text.replace_range(range, "");
        }
        self.finish_operator(start, next_mode);
        if let Some(mut offsets) = block_offsets {
            let origin = offsets.remove(0);
            self.block_insert = Some(super::BlockInsert {
                origin,
                targets: offsets,
                text: String::new(),
            });
        }
        if next_mode == Mode::Normal {
            self.commit_change();
        }
        true
    }

    pub(super) fn apply_motion(&mut self, motion: Motion, next_mode: Mode) -> bool {
        let cursor = self.cursor;
        let Some(range) = self.motion_range(motion) else {
            return false;
        };
        self.cursor = cursor;
        self.apply_range(range, next_mode)
    }

    pub(super) fn motion_range(&mut self, motion: Motion) -> Option<Range<usize>> {
        if self.mode != Mode::Normal {
            return None;
        }
        let start = self.position_to_byte(self.cursor, false);
        match motion {
            Motion::WordForward => self.move_word_forward(),
            Motion::WordBackward => self.move_word_backward(),
            Motion::WordEnd => self.move_word_end(),
            Motion::BigWordForward => self.move_big_word_forward(),
            Motion::BigWordBackward => self.move_big_word_backward(),
            Motion::BigWordEnd => self.move_big_word_end(),
        }
        let target = self.position_to_byte(self.cursor, false);
        let range = match motion {
            Motion::WordForward | Motion::BigWordForward
                if self.position_from_byte(start).line != self.position_from_byte(target).line =>
            {
                start..self.line_bounds(self.position_from_byte(start).line).1
            }
            Motion::WordForward | Motion::BigWordForward => start..target,
            Motion::WordBackward | Motion::BigWordBackward => target..start,
            Motion::WordEnd | Motion::BigWordEnd => start..self.next_grapheme_boundary(target),
        };
        (!range.is_empty()).then_some(range)
    }

    pub(super) fn text_object_range(&self, object: TextObject) -> Option<Range<usize>> {
        if self.mode != Mode::Normal || self.text.is_empty() {
            return None;
        }
        let big = matches!(object, TextObject::InnerBigWord | TextObject::ABigWord);
        let around = matches!(object, TextObject::AWord | TextObject::ABigWord);
        let graphemes = self
            .text
            .grapheme_indices(true)
            .map(|(byte, grapheme)| (byte, grapheme.len(), word_class(grapheme, big)))
            .collect::<Vec<_>>();
        let cursor = self.position_to_byte(self.cursor, false);
        let index = graphemes
            .partition_point(|(byte, _, _)| *byte < cursor)
            .min(graphemes.len() - 1);
        let class = graphemes[index].2;
        if class == WordClass::LineBreak {
            return None;
        }
        let mut first = index;
        let mut last = index;
        while first > 0 && graphemes[first - 1].2 == class {
            first -= 1;
        }
        while last + 1 < graphemes.len() && graphemes[last + 1].2 == class {
            last += 1;
        }
        if around {
            if last + 1 < graphemes.len() && graphemes[last + 1].2 == WordClass::HorizontalSpace {
                while last + 1 < graphemes.len()
                    && graphemes[last + 1].2 == WordClass::HorizontalSpace
                {
                    last += 1;
                }
            } else {
                while first > 0 && graphemes[first - 1].2 == WordClass::HorizontalSpace {
                    first -= 1;
                }
            }
        }
        Some(graphemes[first].0..graphemes[last].0 + graphemes[last].1)
    }

    pub(super) fn apply_range(&mut self, range: Range<usize>, next_mode: Mode) -> bool {
        if range.is_empty() {
            return false;
        }
        self.begin_change();
        let start = range.start;
        self.unnamed_register = Register {
            text: self.text[range.clone()].to_owned(),
            linewise: false,
        };
        self.text.replace_range(range, "");
        self.finish_operator(start, next_mode);
        if next_mode == Mode::Normal {
            self.commit_change();
        }
        true
    }

    pub(super) fn yank_range(&mut self, range: Range<usize>, cursor: usize) -> bool {
        if range.is_empty() {
            return false;
        }
        self.unnamed_register = Register {
            text: self.text[range].to_owned(),
            linewise: false,
        };
        self.clear_selection(Mode::Normal);
        self.cursor = self.normal_position_from_byte(cursor.min(self.text.len()));
        self.preferred_display_column = None;
        true
    }

    pub(super) fn can_apply_linewise(&self) -> bool {
        self.kind == BufferKind::Textarea && self.mode == Mode::Normal
    }

    pub(super) fn selected_line_interval(&self) -> (usize, usize) {
        let anchor = self.anchor.unwrap_or(self.cursor);
        (
            anchor.line.min(self.cursor.line),
            anchor.line.max(self.cursor.line),
        )
    }

    pub(super) fn linewise_text(&self, first: usize, last: usize) -> String {
        if self.text.is_empty() {
            return String::new();
        }
        let start = self.line_bounds(first).0;
        let end = self.line_bounds(last).1;
        let mut text = self.text[start..end].to_owned();
        text.push('\n');
        text
    }

    pub(super) fn line_delete_range(&self, first: usize, last: usize) -> Range<usize> {
        let start = self.line_bounds(first).0;
        let end = self.line_bounds(last).1;
        if end < self.text.len() {
            start..end + 1
        } else if start > 0 {
            start - 1..end
        } else {
            start..end
        }
    }

    fn apply_visual_line_operator(&mut self, next_mode: Mode) -> bool {
        let (first, last) = self.selected_line_interval();
        let selected_start = self.line_bounds(first).0;
        self.unnamed_register = Register {
            text: self.linewise_text(first, last),
            linewise: true,
        };
        let range = self.line_delete_range(first, last);
        let start = range.start;
        let consumed_preceding_separator = start < selected_start;
        self.text.replace_range(range, "");
        let mut cursor = start;
        if next_mode == Mode::Insert {
            let insertion = start.min(self.text.len());
            if !self.text.is_empty() {
                self.text.insert(insertion, '\n');
                if consumed_preceding_separator {
                    cursor = insertion + 1;
                }
            }
        }
        self.finish_operator(cursor, next_mode);
        if next_mode == Mode::Normal {
            self.commit_change();
        }
        true
    }

    pub(super) fn paste(&mut self, before: bool) -> bool {
        if self.mode != Mode::Normal || self.unnamed_register.text.is_empty() {
            return false;
        }
        self.begin_change();
        let register = self.unnamed_register.clone();
        if register.linewise && self.kind == BufferKind::Textarea {
            let content = register.text.strip_suffix('\n').unwrap_or(&register.text);
            let (line_start, line_end) = self.line_bounds(self.cursor.line);
            let (insertion, value, cursor) = if before {
                (line_start, format!("{content}\n"), line_start)
            } else {
                (line_end, format!("\n{content}"), line_end + 1)
            };
            self.text.insert_str(insertion, &value);
            self.cursor = self.normal_position_from_byte(cursor);
        } else {
            let cursor = self.position_to_byte(self.cursor, false);
            let insertion = if before {
                cursor
            } else {
                self.next_grapheme_boundary(cursor)
            };
            self.text.insert_str(insertion, &register.text);
            let end = insertion + register.text.len();
            self.cursor = self.normal_position_from_byte(self.previous_grapheme_boundary(end));
        }
        self.preferred_display_column = None;
        self.commit_change();
        true
    }

    pub(super) fn cursor_is_whitespace(&self, big: bool) -> bool {
        let cursor = self.position_to_byte(self.cursor, false);
        self.text[cursor..]
            .graphemes(true)
            .next()
            .is_none_or(|grapheme| word_class(grapheme, big).is_whitespace())
    }

    pub(super) fn change_to_current_word_end(&mut self, big: bool) -> bool {
        if self.mode != Mode::Normal {
            return false;
        }
        let start = self.position_to_byte(self.cursor, false);
        let Some(first) = self.text[start..].graphemes(true).next() else {
            return false;
        };
        let class = word_class(first, big);
        let mut end = start;
        for (relative, grapheme) in self.text[start..].grapheme_indices(true) {
            if word_class(grapheme, big) != class {
                break;
            }
            end = start + relative + grapheme.len();
        }
        self.apply_range(start..end, Mode::Insert)
    }

    fn finish_operator(&mut self, start: usize, next_mode: Mode) {
        self.clear_selection(next_mode);
        self.cursor = if next_mode == Mode::Insert {
            self.position_from_byte(start.min(self.text.len()))
        } else {
            self.normal_position_from_byte(start.min(self.text.len()))
        };
        self.insert_origin = (next_mode == Mode::Insert).then_some(self.cursor);
        self.insert_advanced = false;
        self.preferred_display_column = None;
    }
}
