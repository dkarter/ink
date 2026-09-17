use unicode_segmentation::UnicodeSegmentation;

use super::{BufferKind, Editor, Mode};

impl Editor {
    pub(super) fn move_vertical(&mut self, direction: isize) {
        if self.kind != BufferKind::Textarea {
            return;
        }
        let display_column = self
            .preferred_display_column
            .unwrap_or_else(|| self.display_column(self.cursor));
        self.preferred_display_column = Some(display_column);
        let last_line = self.line_count().saturating_sub(1);
        let line = self
            .cursor
            .line
            .saturating_add_signed(direction)
            .min(last_line);
        self.cursor = self.position_at_display_column(line, display_column);
    }

    pub(super) fn move_word(&mut self, forward: bool, big: bool) {
        let graphemes = self
            .text
            .grapheme_indices(true)
            .map(|(byte, grapheme)| (byte, word_class(grapheme, big)))
            .collect::<Vec<_>>();
        if graphemes.is_empty() {
            return;
        }
        let cursor_byte = self.position_to_byte(self.cursor, self.mode == Mode::Insert);
        let mut index = graphemes
            .partition_point(|(byte, _)| *byte < cursor_byte)
            .min(graphemes.len() - 1);
        if forward {
            let class = graphemes[index].1;
            if !class.is_whitespace() {
                while index < graphemes.len() && graphemes[index].1 == class {
                    index += 1;
                }
            }
            while index < graphemes.len() && graphemes[index].1.is_whitespace() {
                index += 1;
            }
            index = index.min(graphemes.len() - 1);
        } else if index > 0 {
            index -= 1;
            while index > 0 && graphemes[index].1.is_whitespace() {
                index -= 1;
            }
            let class = graphemes[index].1;
            while index > 0 && graphemes[index - 1].1 == class {
                index -= 1;
            }
        }
        self.cursor = self.normal_position_from_byte(graphemes[index].0);
        self.preferred_display_column = None;
    }

    pub(super) fn move_word_end_with(&mut self, big: bool) {
        let graphemes = self
            .text
            .grapheme_indices(true)
            .map(|(byte, grapheme)| (byte, word_class(grapheme, big)))
            .collect::<Vec<_>>();
        if graphemes.is_empty() {
            return;
        }
        let cursor_byte = self.position_to_byte(self.cursor, self.mode == Mode::Insert);
        let mut index = graphemes
            .partition_point(|(byte, _)| *byte < cursor_byte)
            .min(graphemes.len() - 1);
        let class = graphemes[index].1;
        let at_end = !class.is_whitespace()
            && (index + 1 == graphemes.len() || graphemes[index + 1].1 != class);
        if class.is_whitespace() || at_end {
            index = (index + usize::from(at_end)).min(graphemes.len() - 1);
            while index < graphemes.len() - 1 && graphemes[index].1.is_whitespace() {
                index += 1;
            }
        }
        let class = graphemes[index].1;
        while index < graphemes.len() - 1 && graphemes[index + 1].1 == class {
            index += 1;
        }
        self.cursor = self.normal_position_from_byte(graphemes[index].0);
        self.preferred_display_column = None;
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum WordClass {
    HorizontalSpace,
    LineBreak,
    Keyword,
    Punctuation,
}

impl WordClass {
    pub(super) const fn is_whitespace(self) -> bool {
        matches!(self, Self::HorizontalSpace | Self::LineBreak)
    }
}

pub(super) fn word_class(grapheme: &str, big: bool) -> WordClass {
    if grapheme == "\n" {
        WordClass::LineBreak
    } else if grapheme.chars().all(char::is_whitespace) {
        WordClass::HorizontalSpace
    } else if big
        || grapheme
            .chars()
            .next()
            .is_some_and(|character| character.is_alphanumeric() || character == '_')
    {
        WordClass::Keyword
    } else {
        WordClass::Punctuation
    }
}
