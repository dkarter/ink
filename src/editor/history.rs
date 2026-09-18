use std::mem;

use super::{Editor, Mode, Position};

const HISTORY_LIMIT: usize = 100;
const HISTORY_BYTE_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct History {
    undo: Vec<Action>,
    redo: Vec<Action>,
    pending: Option<Snapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Action {
    before: Snapshot,
    after: Snapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Snapshot {
    text: String,
    cursor: Position,
}

impl Editor {
    pub(super) fn begin_change(&mut self) {
        if self.history.pending.is_none() {
            self.history.pending = Some(Snapshot {
                text: self.text.clone(),
                cursor: self.cursor,
            });
        }
    }

    pub(super) fn commit_change(&mut self) {
        let Some(before) = self.history.pending.take() else {
            return;
        };
        if before.text == self.text {
            return;
        }
        self.history.redo.clear();
        let action = Action {
            before,
            after: Snapshot {
                text: String::new(),
                cursor: self.cursor,
            },
        };
        push_bounded(&mut self.history.undo, action);
    }

    pub fn undo(&mut self) -> bool {
        if self.mode != Mode::Normal {
            return false;
        }
        let Some(mut action) = self.history.undo.pop() else {
            return false;
        };
        action.after.text = mem::take(&mut self.text);
        self.text = mem::take(&mut action.before.text);
        let cursor = action.before.cursor;
        push_bounded(&mut self.history.redo, action);
        self.restore_history_cursor(cursor);
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.mode != Mode::Normal {
            return false;
        }
        let Some(mut action) = self.history.redo.pop() else {
            return false;
        };
        action.before.text = mem::take(&mut self.text);
        self.text = mem::take(&mut action.after.text);
        let cursor = action.after.cursor;
        push_bounded(&mut self.history.undo, action);
        self.restore_history_cursor(cursor);
        true
    }

    fn restore_history_cursor(&mut self, cursor: Position) {
        self.history.pending = None;
        self.clear_selection(Mode::Normal);
        self.cursor = self.normalized_position(cursor);
        self.preferred_display_column = None;
    }
}

fn push_bounded(history: &mut Vec<Action>, action: Action) {
    history.push(action);
    while history.len() > HISTORY_LIMIT || history_bytes(history) > HISTORY_BYTE_LIMIT {
        history.remove(0);
    }
}

fn history_bytes(history: &[Action]) -> usize {
    history
        .iter()
        .map(|action| action.before.text.len() + action.after.text.len())
        .sum()
}
