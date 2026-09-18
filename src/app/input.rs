use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    cli::PromptKind,
    editor::{Editor, Mode, Motion, TextObject},
    terminal::{CancelReason, PromptOutcome},
};

pub(super) fn handle_key(
    editor: &mut Editor,
    kind: PromptKind,
    key: KeyEvent,
    pending: &mut Option<Pending>,
) -> Option<PromptOutcome> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Some(PromptOutcome::Cancelled(CancelReason::Interrupt)),
            KeyCode::Char('d') => {
                editor.escape();
                Some(PromptOutcome::Accepted(editor.text().to_owned()))
            }
            KeyCode::Char('v') if editor.mode() == Mode::Normal => {
                editor.enter_visual_block();
                None
            }
            _ => None,
        };
    }
    if editor.mode() == Mode::Normal && pending.is_some() {
        handle_pending(editor, key.code, pending);
        return None;
    }
    match editor.mode() {
        Mode::Insert => match key.code {
            KeyCode::Esc => editor.escape(),
            KeyCode::Left => editor.move_left(),
            KeyCode::Right => editor.move_right(),
            KeyCode::Up => editor.move_up(),
            KeyCode::Down => editor.move_down(),
            KeyCode::Home => editor.move_to_line_start(),
            KeyCode::End => editor.move_to_line_end(),
            KeyCode::Backspace => {
                editor.backspace();
            }
            KeyCode::Enter if kind == PromptKind::Input => {
                return Some(PromptOutcome::Accepted(editor.text().to_owned()));
            }
            KeyCode::Enter => {
                editor.insert("\n");
            }
            KeyCode::Tab => {
                editor.insert("\t");
            }
            KeyCode::Char(character) => {
                editor.insert(&character.to_string());
            }
            _ => {}
        },
        Mode::Normal => match key.code {
            KeyCode::Char('q') => return Some(PromptOutcome::Cancelled(CancelReason::NormalQuit)),
            KeyCode::Enter => return Some(PromptOutcome::Accepted(editor.text().to_owned())),
            KeyCode::Char('i') => editor.enter_insert(),
            KeyCode::Char('A') => {
                editor.append_at_line_end();
            }
            KeyCode::Char('u') => {
                editor.undo();
            }
            KeyCode::Char('r') => {
                editor.redo();
            }
            KeyCode::Char('v') => editor.enter_visual(),
            KeyCode::Char('V') => editor.enter_visual_line(),
            KeyCode::Char('d') => *pending = Some(Pending::Operator(Operator::Delete)),
            KeyCode::Char('c') => *pending = Some(Pending::Operator(Operator::Change)),
            KeyCode::Char('y') => *pending = Some(Pending::Operator(Operator::Yank)),
            KeyCode::Char('p') => {
                editor.paste_after();
            }
            KeyCode::Char('P') => {
                editor.paste_before();
            }
            KeyCode::Char('o') => {
                editor.open_line_below();
            }
            KeyCode::Char('O') => {
                editor.open_line_above();
            }
            KeyCode::Char('x') => {
                editor.delete_at_cursor();
            }
            code => move_cursor(editor, code),
        },
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => match key.code {
            KeyCode::Esc => editor.escape(),
            KeyCode::Char('d') | KeyCode::Char('x') => {
                editor.delete_selection();
            }
            KeyCode::Char('c') => {
                editor.change_selection();
            }
            KeyCode::Char('y') => {
                editor.yank_selection();
            }
            code => move_cursor(editor, code),
        },
    }
    None
}

#[derive(Clone, Copy)]
pub(super) enum Pending {
    Operator(Operator),
    TextObject { operator: Operator, around: bool },
}

#[derive(Clone, Copy)]
pub(super) enum Operator {
    Delete,
    Change,
    Yank,
}

fn handle_pending(editor: &mut Editor, code: KeyCode, pending: &mut Option<Pending>) {
    let current = pending.take().expect("pending state was checked");
    match (current, code) {
        (Pending::Operator(Operator::Delete), KeyCode::Char('d')) => {
            editor.delete_line();
        }
        (Pending::Operator(Operator::Change), KeyCode::Char('c')) => {
            editor.change_line();
        }
        (Pending::Operator(Operator::Yank), KeyCode::Char('y')) => {
            editor.yank_line();
        }
        (Pending::Operator(operator), KeyCode::Char('i')) => {
            *pending = Some(Pending::TextObject {
                operator,
                around: false,
            });
        }
        (Pending::Operator(operator), KeyCode::Char('a')) => {
            *pending = Some(Pending::TextObject {
                operator,
                around: true,
            });
        }
        (Pending::Operator(operator), code) => {
            if let Some(motion) = motion_for_key(code) {
                apply_motion(editor, operator, motion);
            }
        }
        (Pending::TextObject { operator, around }, KeyCode::Char('w')) => {
            apply_text_object(
                editor,
                operator,
                if around {
                    TextObject::AWord
                } else {
                    TextObject::InnerWord
                },
            );
        }
        (Pending::TextObject { operator, around }, KeyCode::Char('W')) => {
            apply_text_object(
                editor,
                operator,
                if around {
                    TextObject::ABigWord
                } else {
                    TextObject::InnerBigWord
                },
            );
        }
        _ => {}
    }
}

fn motion_for_key(code: KeyCode) -> Option<Motion> {
    match code {
        KeyCode::Char('w') => Some(Motion::WordForward),
        KeyCode::Char('b') => Some(Motion::WordBackward),
        KeyCode::Char('e') => Some(Motion::WordEnd),
        KeyCode::Char('W') => Some(Motion::BigWordForward),
        KeyCode::Char('B') => Some(Motion::BigWordBackward),
        KeyCode::Char('E') => Some(Motion::BigWordEnd),
        _ => None,
    }
}

fn apply_motion(editor: &mut Editor, operator: Operator, motion: Motion) {
    match operator {
        Operator::Delete => editor.delete_motion(motion),
        Operator::Change => editor.change_motion(motion),
        Operator::Yank => editor.yank_motion(motion),
    };
}

fn apply_text_object(editor: &mut Editor, operator: Operator, object: TextObject) {
    match operator {
        Operator::Delete => editor.delete_text_object(object),
        Operator::Change => editor.change_text_object(object),
        Operator::Yank => editor.yank_text_object(object),
    };
}

fn move_cursor(editor: &mut Editor, code: KeyCode) {
    match code {
        KeyCode::Left | KeyCode::Char('h') => editor.move_left(),
        KeyCode::Down | KeyCode::Char('j') => editor.move_down(),
        KeyCode::Up | KeyCode::Char('k') => editor.move_up(),
        KeyCode::Right | KeyCode::Char('l') => editor.move_right(),
        KeyCode::Home | KeyCode::Char('0') => editor.move_to_line_start(),
        KeyCode::End | KeyCode::Char('$') => editor.move_to_line_end(),
        KeyCode::Char('w') => editor.move_word_forward(),
        KeyCode::Char('b') => editor.move_word_backward(),
        KeyCode::Char('W') => editor.move_big_word_forward(),
        KeyCode::Char('B') => editor.move_big_word_backward(),
        KeyCode::Char('e') => editor.move_word_end(),
        KeyCode::Char('E') => editor.move_big_word_end(),
        _ => {}
    }
}
