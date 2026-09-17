use std::borrow::Cow;

use ratatui::buffer::CellWidth;
use unicode_segmentation::UnicodeSegmentation;

const TAB_STOP: usize = 4;

pub(crate) fn text_width(text: &str) -> usize {
    text.graphemes(true).fold(0, |column, grapheme| {
        column + grapheme_width_at(grapheme, column)
    })
}

pub(crate) fn grapheme_width_at(grapheme: &str, column: usize) -> usize {
    if grapheme == "\t" {
        return TAB_STOP - column % TAB_STOP;
    }
    if grapheme.contains(char::is_control) {
        0
    } else {
        usize::from(grapheme.cell_width())
    }
}

pub(crate) fn cursor_width(text: &str, column: usize) -> usize {
    let mut cell = 0;
    for (index, grapheme) in text.graphemes(true).enumerate() {
        let width = grapheme_width_at(grapheme, cell);
        if index == column {
            return width.max(1);
        }
        cell += width;
    }
    1
}

pub(crate) fn sanitized_prefix(text: &str, start_cell: usize, max_width: usize) -> Cow<'_, str> {
    let mut visible_width = 0;
    let mut visible_end = 0;
    let mut sanitized = None::<String>;

    for (byte, grapheme) in text.grapheme_indices(true) {
        if visible_width == max_width {
            break;
        }
        let width = grapheme_width_at(grapheme, start_cell + visible_width);
        if grapheme == "\t" {
            sanitized
                .get_or_insert_with(|| text[..byte].to_owned())
                .extend(std::iter::repeat_n(' ', width));
            visible_width += width;
            visible_end = byte + grapheme.len();
            continue;
        }
        if width == 0 {
            sanitized.get_or_insert_with(|| text[..byte].to_owned());
            continue;
        }
        if visible_width + width > max_width {
            break;
        }
        visible_width += width;
        visible_end = byte + grapheme.len();
        if let Some(output) = &mut sanitized {
            output.push_str(grapheme);
        }
    }

    sanitized.map_or_else(|| Cow::Borrowed(&text[..visible_end]), Cow::Owned)
}

pub(crate) fn slice_from_cell(text: &str, offset: usize) -> (&str, usize) {
    let mut position = 0;
    for (byte, grapheme) in text.grapheme_indices(true) {
        if position >= offset {
            return (&text[byte..], position - offset);
        }
        position += grapheme_width_at(grapheme, position);
    }
    ("", 0)
}

pub(crate) fn boundary_at_or_after(text: &str, target: usize) -> usize {
    let mut boundary = 0;
    for grapheme in text.graphemes(true) {
        if boundary >= target {
            break;
        }
        boundary += grapheme_width_at(grapheme, boundary);
    }
    boundary
}
