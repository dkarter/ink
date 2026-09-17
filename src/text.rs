//! Prompt-specific text normalization.

use std::borrow::Cow;

pub(crate) fn single_line(text: &str) -> String {
    text.chars()
        .filter(|character| !matches!(character, '\r' | '\n'))
        .collect()
}

pub(crate) fn textarea(text: &str) -> Cow<'_, str> {
    if text.contains('\r') {
        Cow::Owned(normalize_textarea(text))
    } else {
        Cow::Borrowed(text)
    }
}

pub(crate) fn textarea_owned(text: String) -> String {
    if text.contains('\r') {
        normalize_textarea(&text)
    } else {
        text
    }
}

fn normalize_textarea(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\r' {
            if characters.peek() == Some(&'\n') {
                characters.next();
            }
            normalized.push('\n');
        } else {
            normalized.push(character);
        }
    }
    normalized
}
