use ink::editor::{Editor, InputError, Mode, Motion, Position, TextObject};
use ink::ui::{Textarea, TextareaState};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

#[test]
fn edit_001_enter_and_leave_insert_mode() {
    let mut editor = Editor::input("ab").expect("single-line fixture");
    editor.set_cursor(Position::new(0, 1));

    editor.enter_insert();
    editor.escape();
    assert_eq!(editor.cursor(), Position::new(0, 1));

    let mut navigation = Editor::input("abc").expect("single-line fixture");
    navigation.enter_insert();
    navigation.move_right();
    navigation.escape();
    assert_eq!(navigation.cursor(), Position::new(0, 1));

    editor.enter_insert();
    assert!(editor.insert("e\u{301}"));
    editor.escape();

    assert_eq!(editor.text(), "ae\u{301}b");
    assert_eq!(editor.mode(), Mode::Normal);
    assert_eq!(editor.cursor(), Position::new(0, 1));

    let mut textarea = Editor::textarea("a");
    textarea.enter_insert();
    textarea.move_right();
    assert!(textarea.insert("\n"));
    textarea.escape();
    assert_eq!(textarea.cursor(), Position::new(1, 0));

    let mut normalized = Editor::textarea("a\r\nb\rc\nd");
    assert_eq!(normalized.text(), "a\nb\nc\nd");
    normalized.enter_insert();
    assert!(normalized.insert("x\r\ny\rz"));
    assert!(!normalized.text().contains('\r'));

    assert_eq!(Editor::input("first\nsecond"), Err(InputError::LineBreak));
    assert_eq!(Editor::empty_input().text(), "");
}

#[test]
fn edit_002_select_characters_visually() {
    let mut editor = Editor::input("abcd").expect("single-line fixture");
    editor.set_cursor(Position::new(0, 1));
    editor.enter_visual();
    editor.move_right();
    editor.move_right();

    assert_eq!(editor.selected_text(), "bcd");
    assert_eq!(editor.selection_ranges(), vec![1..4]);
}

#[test]
fn edit_003_select_whole_lines() {
    let mut editor = Editor::textarea("zero\none\ntwo\nthree");
    editor.set_cursor(Position::new(1, 1));
    editor.enter_visual_line();
    editor.move_down();

    assert_eq!(editor.selected_text(), "one\ntwo\n");
    assert_eq!(editor.selection_ranges(), vec![5..13]);
}

#[test]
fn edit_004_select_a_text_column() {
    let mut editor = Editor::textarea("a界z\n12\nq界rst");
    editor.set_cursor(Position::new(0, 1));
    editor.enter_visual_block();
    editor.move_down();
    editor.move_down();

    assert_eq!(editor.selected_text(), "界\n2\n界");
    assert_eq!(editor.selection_ranges(), vec![1..4, 7..8, 10..13]);
}

#[test]
fn edit_005_delete_selected_text() {
    let mut editor = Editor::input("abcd").expect("single-line fixture");
    editor.set_cursor(Position::new(0, 1));
    editor.enter_visual();
    editor.move_right();

    assert!(editor.delete_selection());
    assert_eq!(editor.text(), "ad");
    assert_eq!(editor.unnamed_register(), "bc");
    assert_eq!(editor.mode(), Mode::Normal);
    assert_eq!(editor.cursor(), Position::new(0, 1));
}

#[test]
fn edit_006_change_selected_text() {
    let mut editor = Editor::input("abcd").expect("single-line fixture");
    editor.set_cursor(Position::new(0, 2));
    editor.enter_visual();
    editor.move_left();

    assert!(editor.change_selection());
    assert_eq!(editor.text(), "ad");
    assert_eq!(editor.unnamed_register(), "bc");
    assert_eq!(editor.mode(), Mode::Insert);
    assert_eq!(editor.cursor(), Position::new(0, 1));
    assert!(editor.insert("XY"));
    assert_eq!(editor.text(), "aXYd");

    let mut final_line = Editor::textarea("one\ntwo");
    final_line.set_cursor(Position::new(1, 0));
    final_line.enter_visual_line();
    assert!(final_line.change_selection());
    assert_eq!(final_line.text(), "one\n");
    assert_eq!(final_line.cursor(), Position::new(1, 0));
    assert_eq!(final_line.unnamed_register(), "two\n");
    assert!(final_line.insert("new"));
    assert_eq!(final_line.text(), "one\nnew");

    let mut tail = Editor::textarea("one\ntwo\nthree");
    tail.set_cursor(Position::new(1, 0));
    tail.enter_visual_line();
    tail.move_down();
    assert!(tail.change_selection());
    assert_eq!(tail.text(), "one\n");
    assert_eq!(tail.cursor(), Position::new(1, 0));
    assert_eq!(tail.unnamed_register(), "two\nthree\n");
    assert!(tail.insert("tail"));
    assert_eq!(tail.text(), "one\ntail");
}

#[test]
fn edit_007_yank_selected_text() {
    let mut editor = Editor::textarea("first\nsecond\nthird");
    editor.set_cursor(Position::new(2, 3));
    editor.enter_visual_line();
    editor.move_up();
    let original = editor.text().to_owned();

    assert!(editor.yank_selection());
    assert_eq!(editor.text(), original);
    assert_eq!(editor.unnamed_register(), "second\nthird\n");
    assert_eq!(editor.mode(), Mode::Normal);
    assert_eq!(editor.cursor(), Position::new(1, 0));
}

#[test]
fn edit_008_block_operators_preserve_rows() {
    let mut deleted = block_editor();
    assert!(deleted.delete_selection());
    assert_eq!(deleted.text(), "ad\nx\nps");
    assert_eq!(deleted.unnamed_register(), "bc\n\nqr");

    let mut changed = block_editor();
    assert!(changed.change_selection());
    assert_eq!(changed.text(), "ad\nx\nps");
    assert_eq!(changed.mode(), Mode::Insert);

    let mut yanked = block_editor();
    assert!(yanked.yank_selection());
    assert_eq!(yanked.text(), "abcd\nx\npqrs");
    assert_eq!(yanked.unnamed_register(), "bc\n\nqr");
}

#[test]
fn edit_021_change_text_across_a_visual_block() {
    let mut editor = block_editor();
    assert!(editor.change_selection());
    assert!(editor.insert("界x"));
    assert!(editor.backspace());
    assert!(editor.insert("e\u{301}"));
    editor.escape();

    assert_eq!(editor.text(), "a界e\u{301}d\nx界e\u{301}\np界e\u{301}s");
    assert_eq!(editor.mode(), Mode::Normal);
    assert_eq!(editor.cursor(), Position::new(0, 2));

    let mut reversed = Editor::textarea("abcd\nx\npqrs");
    reversed.set_cursor(Position::new(2, 2));
    reversed.enter_visual_block();
    reversed.move_up();
    reversed.move_up();
    reversed.move_left();
    assert!(reversed.change_selection());
    assert!(reversed.insert("R"));
    reversed.escape();
    assert_eq!(reversed.text(), "aRd\nxR\npRs");

    let mut moved = block_editor();
    assert!(moved.change_selection());
    assert!(moved.insert("Q"));
    moved.move_left();
    moved.escape();
    assert_eq!(moved.text(), "aQd\nx\nps");

    let result = super::command_line::prompt_with_delayed_keys(
        "exec {ink} textarea --normal --value 'abcd\nx\npqrs'",
        "\u{16}jjlcXY\u{1b}".as_bytes(),
        std::time::Duration::from_millis(50),
        b"\x04",
        None,
    );
    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"XYcd\nXY\nXYrs\n");

    let accepted = super::command_line::prompt(
        "exec {ink} textarea --normal --value 'abcd\nx\npqrs'",
        "\u{16}jjlcZ\u{4}".as_bytes(),
    );
    assert_eq!(accepted.status, 0);
    assert_eq!(accepted.stdout, b"Zcd\nZ\nZrs\n");
}

#[test]
fn edit_022_traverse_editing_history() {
    let mut editor = Editor::input("abc").expect("input fixture");
    assert!(!editor.undo());
    assert!(!editor.redo());

    editor.enter_insert();
    assert!(editor.insert("Xe\u{301}"));
    assert!(editor.insert("👩‍💻"));
    editor.escape();
    assert_eq!(editor.text(), "Xe\u{301}👩‍💻abc");

    assert!(editor.delete_at_cursor());
    assert_eq!(editor.text(), "Xe\u{301}abc");
    assert!(editor.undo());
    assert_eq!(editor.text(), "Xe\u{301}👩‍💻abc");
    assert!(editor.undo());
    assert_eq!(editor.text(), "abc");
    assert_eq!(editor.cursor(), Position::new(0, 0));
    assert!(editor.redo());
    assert_eq!(editor.text(), "Xe\u{301}👩‍💻abc");
    assert!(editor.redo());
    assert_eq!(editor.text(), "Xe\u{301}abc");

    assert!(editor.undo());
    assert!(editor.delete_at_cursor());
    assert_eq!(editor.text(), "Xe\u{301}abc");
    assert!(!editor.redo());

    let mut operator = Editor::input("one two").expect("input fixture");
    assert!(operator.delete_motion(Motion::WordForward));
    assert_eq!(operator.text(), "two");
    assert!(operator.undo());
    assert_eq!(operator.text(), "one two");
    assert!(operator.redo());
    assert_eq!(operator.text(), "two");

    let mut block = block_editor();
    assert!(block.change_selection());
    assert!(block.insert("R"));
    block.escape();
    assert_eq!(block.text(), "aRd\nxR\npRs");
    assert!(block.undo());
    assert_eq!(block.text(), "abcd\nx\npqrs");
    assert!(block.redo());
    assert_eq!(block.text(), "aRd\nxR\npRs");

    let mut append = Editor::input("abc").expect("input fixture");
    assert!(append.append_at_line_end());
    assert!(append.insert("z"));
    append.escape();
    append.move_left();
    assert!(append.undo());
    assert_eq!(append.text(), "abc");
    assert_eq!(append.cursor(), Position::new(0, 0));
    assert!(append.redo());
    assert_eq!(append.text(), "abcz");
    assert_eq!(append.cursor(), Position::new(0, 3));

    let mut no_op = Editor::input("a").expect("input fixture");
    no_op.enter_insert();
    assert!(no_op.insert("X"));
    no_op.escape();
    assert!(no_op.undo());
    no_op.enter_insert();
    assert!(no_op.insert("Y"));
    assert!(no_op.backspace());
    no_op.escape();
    assert!(no_op.redo());
    assert_eq!(no_op.text(), "Xa");

    let mut bounded = Editor::empty_input();
    for _ in 0..=100 {
        bounded.enter_insert();
        assert!(bounded.insert("x"));
        bounded.escape();
    }
    for _ in 0..100 {
        assert!(bounded.undo());
    }
    assert!(!bounded.undo());

    let result = super::command_line::prompt_with_delayed_keys(
        "exec {ink} input --normal --value 'abc'",
        b"iXY\x1b",
        std::time::Duration::from_millis(50),
        b"ur\x04",
        None,
    );
    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"XYabc\n");
}

#[test]
fn edit_009_treat_joined_unicode_as_one_character() {
    let family = "👨‍👩‍👧‍👦";
    let mut editor = Editor::input(format!("Ae\u{301}{family}Z")).expect("single-line fixture");
    editor.move_right();
    editor.enter_visual();
    editor.move_right();

    assert_eq!(editor.selected_text(), format!("e\u{301}{family}"));
    assert!(editor.delete_selection());
    assert_eq!(editor.text(), "AZ");
    assert_eq!(editor.unnamed_register(), format!("e\u{301}{family}"));
}

#[test]
fn edit_010_deleting_at_boundaries_keeps_a_valid_cursor() {
    let mut editor = Editor::input("abc").expect("single-line fixture");
    editor.set_cursor(Position::new(0, 2));
    assert!(editor.delete_at_cursor());
    assert_eq!(editor.text(), "ab");
    assert_eq!(editor.cursor(), Position::new(0, 1));

    editor.set_cursor(Position::new(0, 0));
    assert!(editor.delete_at_cursor());
    assert_eq!(editor.text(), "b");
    assert_eq!(editor.cursor(), Position::new(0, 0));
    assert!(editor.delete_at_cursor());
    assert_eq!(editor.text(), "");
    assert_eq!(editor.cursor(), Position::new(0, 0));

    let mut editor = Editor::textarea("one\ntwo");
    editor.set_cursor(Position::new(1, 0));
    editor.enter_visual_line();
    assert!(editor.delete_selection());
    assert_eq!(editor.text(), "one");
    assert_eq!(editor.cursor(), Position::new(0, 2));

    let mut trailing = Editor::textarea("a\n");
    trailing.move_down();
    assert_eq!(trailing.cursor(), Position::new(1, 0));
    trailing.set_cursor(Position::new(1, 0));
    assert_eq!(trailing.cursor(), Position::new(1, 0));

    let mut intermediate = Editor::textarea("a\n\nb");
    intermediate.move_down();
    assert_eq!(intermediate.cursor(), Position::new(1, 0));
    intermediate.set_cursor(Position::new(1, 0));
    assert_eq!(intermediate.cursor(), Position::new(1, 0));

    trailing.enter_insert();
    trailing.move_down();
    assert_eq!(trailing.cursor(), Position::new(1, 0));

    let mut visual = Editor::textarea("\na");
    assert_eq!(visual.cursor(), Position::new(0, 0));
    visual.enter_visual_line();
    visual.move_down();
    assert_eq!(visual.cursor(), Position::new(1, 0));
    assert!(visual.yank_selection());
    assert_eq!(visual.cursor(), Position::new(0, 0));
}

#[test]
fn edit_011_move_by_word_and_word_boundaries() {
    let mut editor = Editor::input("one two-three  界word").unwrap();
    editor.move_word_forward();
    assert_eq!(editor.cursor(), Position::new(0, 4));
    editor.move_word_forward();
    assert_eq!(editor.cursor(), Position::new(0, 7));
    editor.move_word_forward();
    assert_eq!(editor.cursor(), Position::new(0, 8));
    editor.move_word_backward();
    assert_eq!(editor.cursor(), Position::new(0, 7));
    editor.move_big_word_forward();
    assert_eq!(editor.cursor(), Position::new(0, 15));
    editor.move_big_word_backward();
    assert_eq!(editor.cursor(), Position::new(0, 4));

    editor.enter_visual();
    editor.move_big_word_forward();
    assert_eq!(editor.selected_text(), "two-three  界");

    let mut ends = Editor::input("one two-three").unwrap();
    ends.move_word_end();
    assert_eq!(ends.cursor(), Position::new(0, 2));
    ends.move_word_end();
    assert_eq!(ends.cursor(), Position::new(0, 6));
    ends.move_word_end();
    assert_eq!(ends.cursor(), Position::new(0, 7));
    ends.move_word_end();
    assert_eq!(ends.cursor(), Position::new(0, 12));

    let mut big_ends = Editor::input("one two-three").unwrap();
    big_ends.move_big_word_end();
    assert_eq!(big_ends.cursor(), Position::new(0, 2));
    big_ends.move_big_word_end();
    assert_eq!(big_ends.cursor(), Position::new(0, 12));

    let mut visual_end = Editor::input("one two").unwrap();
    visual_end.enter_visual();
    visual_end.move_word_end();
    assert_eq!(visual_end.selected_text(), "one");

    for keys in [b"wwbx\r".as_slice(), b"WWBx\r".as_slice()] {
        let result =
            super::command_line::prompt("exec {ink} input --normal --value 'one two-three'", keys);
        assert_eq!(result.status, 0);
        assert_eq!(result.stdout, b"one wo-three\n");
    }

    for (keys, expected) in [
        (b"eeex\r".as_slice(), b"one twothree\n".as_slice()),
        (b"EEx\r".as_slice(), b"one two-thre\n".as_slice()),
    ] {
        let result =
            super::command_line::prompt("exec {ink} input --normal --value 'one two-three'", keys);
        assert_eq!(result.status, 0);
        assert_eq!(result.stdout, expected);
    }
}

#[test]
fn edit_012_delete_or_change_a_motion_range() {
    let mut deleted = Editor::input("one two").unwrap();
    assert!(deleted.delete_motion(Motion::WordForward));
    assert_eq!(deleted.text(), "two");
    assert_eq!(deleted.unnamed_register(), "one ");
    assert_eq!(deleted.mode(), Mode::Normal);

    let mut inner = Editor::input("one two").unwrap();
    assert!(inner.delete_text_object(TextObject::InnerWord));
    assert_eq!(inner.text(), " two");

    let mut around = Editor::input("one two").unwrap();
    assert!(around.delete_text_object(TextObject::AWord));
    assert_eq!(around.text(), "two");

    let mut changed = Editor::input("one two").unwrap();
    assert!(changed.change_text_object(TextObject::InnerWord));
    assert_eq!(changed.text(), " two");
    assert_eq!(changed.mode(), Mode::Insert);
    assert!(changed.insert("new"));
    assert_eq!(changed.text(), "new two");

    let mut cross_line = Editor::textarea("one\ntwo");
    assert!(cross_line.delete_motion(Motion::WordForward));
    assert_eq!(cross_line.text(), "\ntwo");

    let mut change_word = Editor::input("one two").unwrap();
    assert!(change_word.change_motion(Motion::WordForward));
    assert_eq!(change_word.text(), " two");

    let mut change_middle = Editor::input("one two").unwrap();
    change_middle.set_cursor(Position::new(0, 1));
    assert!(change_middle.change_motion(Motion::WordForward));
    assert_eq!(change_middle.text(), "o two");

    let mut change_end = Editor::input("one two").unwrap();
    change_end.set_cursor(Position::new(0, 2));
    assert!(change_end.change_motion(Motion::WordForward));
    assert_eq!(change_end.text(), "on two");

    let mut change_big_end = Editor::input("one-two three").unwrap();
    change_big_end.set_cursor(Position::new(0, 6));
    assert!(change_big_end.change_motion(Motion::BigWordForward));
    assert_eq!(change_big_end.text(), "one-tw three");

    let mut change_whitespace = Editor::input("one   two").unwrap();
    change_whitespace.move_word_end();
    change_whitespace.move_right();
    assert!(change_whitespace.change_motion(Motion::WordForward));
    assert_eq!(change_whitespace.text(), "onetwo");

    let mut whitespace = Editor::input("one   two").unwrap();
    whitespace.move_word_end();
    whitespace.move_right();
    assert!(whitespace.delete_text_object(TextObject::InnerWord));
    assert_eq!(whitespace.text(), "onetwo");

    let mut multiline_space = Editor::textarea("one  \n  two");
    multiline_space.set_cursor(Position::new(0, 3));
    assert!(multiline_space.delete_text_object(TextObject::InnerWord));
    assert_eq!(multiline_space.text(), "one\n  two");

    let mut empty_line_object = Editor::textarea("one\n\ntwo");
    empty_line_object.move_down();
    assert!(!empty_line_object.delete_text_object(TextObject::InnerWord));
    assert_eq!(empty_line_object.text(), "one\n\ntwo");

    let mut inner_big = Editor::input("one-two three").unwrap();
    assert!(inner_big.delete_text_object(TextObject::InnerBigWord));
    assert_eq!(inner_big.text(), " three");

    let mut around_big = Editor::input("one-two  three").unwrap();
    assert!(around_big.delete_text_object(TextObject::ABigWord));
    assert_eq!(around_big.text(), "three");

    let mut backward = Editor::input("one two").unwrap();
    backward.move_to_line_end();
    assert!(backward.delete_motion(Motion::WordBackward));
    assert_eq!(backward.text(), "one o");

    let mut end = Editor::input("one two").unwrap();
    assert!(end.delete_motion(Motion::WordEnd));
    assert_eq!(end.text(), " two");

    let mut big_end = Editor::input("one-two three").unwrap();
    assert!(big_end.delete_motion(Motion::BigWordEnd));
    assert_eq!(big_end.text(), " three");

    let mut change_backward = Editor::input("one-two three").unwrap();
    change_backward.move_to_line_end();
    assert!(change_backward.change_motion(Motion::BigWordBackward));
    assert_eq!(change_backward.text(), "one-two e");
    assert_eq!(change_backward.mode(), Mode::Insert);

    let mut change_inner_big = Editor::input("one-two three").unwrap();
    assert!(change_inner_big.change_text_object(TextObject::InnerBigWord));
    assert_eq!(change_inner_big.text(), " three");

    let mut decomposed = Editor::input("e\u{301}lan test").unwrap();
    assert!(decomposed.delete_text_object(TextObject::InnerWord));
    assert_eq!(decomposed.text(), " test");

    for (keys, expected) in [
        (b"dw\r".as_slice(), b"two\n".as_slice()),
        (b"diw\r".as_slice(), b" two\n".as_slice()),
        (b"daw\r".as_slice(), b"two\n".as_slice()),
        (b"ciwnew\x04".as_slice(), b"new two\n".as_slice()),
    ] {
        let result =
            super::command_line::prompt("exec {ink} input --normal --value 'one two'", keys);
        assert_eq!(result.status, 0);
        assert_eq!(result.stdout, expected);
    }
}

#[test]
fn edit_013_yank_and_paste_operator_ranges() {
    let mut characters = Editor::input("one two").unwrap();
    assert!(characters.yank_motion(Motion::WordForward));
    assert_eq!(characters.unnamed_register(), "one ");
    characters.move_to_line_end();
    assert!(characters.paste_after());
    assert_eq!(characters.text(), "one twoone ");
    assert!(characters.paste_after());
    assert_eq!(characters.text(), "one twoone one ");

    let mut lines = Editor::textarea("one\ntwo");
    assert!(lines.yank_line());
    lines.move_down();
    assert!(lines.paste_before());
    assert_eq!(lines.text(), "one\none\ntwo");
    assert!(lines.paste_after());
    assert_eq!(lines.text(), "one\none\none\ntwo");

    let mut blank_line = Editor::textarea("one\n\ntwo");
    blank_line.enter_visual_line();
    blank_line.move_down();
    assert!(blank_line.yank_selection());
    blank_line.move_down();
    assert!(blank_line.paste_after());
    assert_eq!(blank_line.text(), "one\n\none\n\ntwo");

    let mut backward = Editor::input("one two").unwrap();
    backward.move_to_line_end();
    assert!(backward.yank_motion(Motion::WordBackward));
    assert_eq!(backward.unnamed_register(), "tw");

    let mut word_end = Editor::input("one two").unwrap();
    assert!(word_end.yank_motion(Motion::WordEnd));
    assert_eq!(word_end.unnamed_register(), "one");

    let mut big_word_end = Editor::input("one-two three").unwrap();
    assert!(big_word_end.yank_motion(Motion::BigWordEnd));
    assert_eq!(big_word_end.unnamed_register(), "one-two");

    let mut inner_big = Editor::input("one-two three").unwrap();
    assert!(inner_big.yank_text_object(TextObject::InnerBigWord));
    assert_eq!(inner_big.unnamed_register(), "one-two");

    let mut big_object = Editor::input("one-two three").unwrap();
    assert!(big_object.yank_text_object(TextObject::ABigWord));
    assert_eq!(big_object.unnamed_register(), "one-two ");
    big_object.move_big_word_forward();
    assert!(big_object.paste_before());
    assert_eq!(big_object.text(), "one-two one-two three");

    let result = super::command_line::prompt(
        "exec {ink} textarea --normal --value 'one\ntwo'",
        b"yyjP\x04",
    );
    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"one\none\ntwo\n");

    for (keys, expected) in [
        (b"yw$p\r".as_slice(), b"one twoone \n".as_slice()),
        (b"yiw$p\r".as_slice(), b"one twoone\n".as_slice()),
    ] {
        let result =
            super::command_line::prompt("exec {ink} input --normal --value 'one two'", keys);
        assert_eq!(result.status, 0);
        assert_eq!(result.stdout, expected);
    }

    let result = super::command_line::prompt(
        "exec {ink} textarea --normal --value 'one\ntwo'",
        b"yyp\x04",
    );
    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"one\none\ntwo\n");
}

#[test]
fn edit_014_apply_linewise_operators() {
    for (line, expected) in [(0, "two\nthree"), (1, "one\nthree"), (2, "one\ntwo")] {
        let mut deleted = Editor::textarea("one\ntwo\nthree");
        deleted.set_cursor(Position::new(line, 0));
        assert!(deleted.delete_line());
        assert_eq!(deleted.text(), expected);
        assert_eq!(deleted.mode(), Mode::Normal);
    }

    for (line, expected, register) in [
        (0, "\ntwo\nthree", "one\n"),
        (1, "one\n\nthree", "two\n"),
        (2, "one\ntwo\n", "three\n"),
    ] {
        let mut changed = Editor::textarea("one\ntwo\nthree");
        changed.set_cursor(Position::new(line, 0));
        assert!(changed.change_line());
        assert_eq!(changed.text(), expected);
        assert_eq!(changed.unnamed_register(), register);
        assert_eq!(changed.mode(), Mode::Insert);
    }

    for (line, register) in [(0, "one\n"), (1, "two\n"), (2, "three\n")] {
        let mut yanked = Editor::textarea("one\ntwo\nthree");
        yanked.set_cursor(Position::new(line, 0));
        assert!(yanked.yank_line());
        assert_eq!(yanked.text(), "one\ntwo\nthree");
        assert_eq!(yanked.unnamed_register(), register);
    }

    let mut empty_line = Editor::textarea("one\n\ntwo");
    empty_line.move_down();
    assert_eq!(empty_line.cursor(), Position::new(1, 0));
    assert!(empty_line.yank_line());
    assert_eq!(empty_line.unnamed_register(), "\n");
    assert!(empty_line.delete_line());
    assert_eq!(empty_line.text(), "one\ntwo");

    let mut empty = Editor::textarea("");
    assert!(empty.change_line());
    assert_eq!(empty.text(), "");
    assert_eq!(empty.mode(), Mode::Insert);

    let mut input = Editor::input("one").unwrap();
    assert!(!input.delete_line());
    assert!(!input.change_line());
    assert!(!input.yank_line());
    input.enter_insert();
    assert!(!input.delete_line());

    let mut wrong_mode = Editor::textarea("one");
    wrong_mode.enter_insert();
    assert!(!wrong_mode.delete_line());
    assert!(!wrong_mode.change_line());
    assert!(!wrong_mode.yank_line());

    let mut visual_mode = Editor::textarea("one");
    visual_mode.enter_visual_line();
    assert!(!visual_mode.delete_line());
    assert!(!visual_mode.change_line());
    assert!(!visual_mode.yank_line());

    for (keys, expected) in [
        (b"dd\x04".as_slice(), b"two\n".as_slice()),
        (b"ccnew\x04".as_slice(), b"new\ntwo\n".as_slice()),
    ] {
        let result =
            super::command_line::prompt("exec {ink} textarea --normal --value 'one\ntwo'", keys);
        assert_eq!(result.status, 0);
        assert_eq!(result.stdout, expected);
    }
}

#[test]
fn edit_016_align_editing_geometry_with_rendered_cells() {
    let mut tabs = Editor::textarea("\tX\n....X");
    tabs.move_right();
    tabs.move_down();
    assert_eq!(tabs.cursor(), Position::new(1, 4));

    let mut block = Editor::textarea("\tX\n....X");
    block.enter_visual_block();
    block.move_down();
    assert_eq!(block.selection_ranges(), vec![0..1, 3..4]);

    let mut controls = Editor::textarea("\u{1}X\nX");
    controls.move_right();
    controls.move_down();
    assert_eq!(controls.cursor(), Position::new(1, 0));

    let mut state = TextareaState::default();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 3));
    Textarea::new(&tabs).render(buffer.area, &mut buffer, &mut state);
    assert_eq!(state.cursor().unwrap().position.x, 4);
    assert_eq!(buffer[(0, 0)].symbol(), " ");
    assert_eq!(buffer[(3, 0)].symbol(), " ");
}

#[test]
fn edit_015_open_a_line_for_insertion() {
    let mut below = Editor::textarea("one\ntwo");
    assert!(below.open_line_below());
    assert_eq!(below.text(), "one\n\ntwo");
    assert_eq!(below.cursor(), Position::new(1, 0));
    assert_eq!(below.mode(), Mode::Insert);

    let mut above = Editor::textarea("one\ntwo");
    above.move_down();
    assert!(above.open_line_above());
    assert_eq!(above.text(), "one\n\ntwo");
    assert_eq!(above.cursor(), Position::new(1, 0));
    assert_eq!(above.mode(), Mode::Insert);

    for (key, expected) in [(b'o', "one\nnew\ntwo\n"), (b'O', "new\none\ntwo\n")] {
        let mut keys = vec![key];
        keys.extend_from_slice(b"new\x04");
        let result =
            super::command_line::prompt("exec {ink} textarea --normal --value 'one\ntwo'", &keys);
        assert_eq!(result.status, 0);
        assert_eq!(String::from_utf8(result.stdout).unwrap(), expected);
    }
}

#[test]
fn edit_017_submit_or_cancel_from_the_command_line() {
    for kind in ["input", "textarea"] {
        let accepted = super::command_line::prompt(
            &format!("exec {{ink}} {kind} --normal --value value"),
            b":\x1b:wq\r",
        );
        assert_eq!(accepted.status, 0, "{kind}");
        assert_eq!(accepted.stdout, b"value\n", "{kind}");
        assert!(contains_in_order(&accepted.terminal, b":wq"));

        let cancelled = super::command_line::prompt(
            &format!("exec {{ink}} {kind} --normal --value unchanged"),
            b":q!\r",
        );
        assert_eq!(cancelled.status, 130, "{kind}");
        assert!(cancelled.stdout.is_empty(), "{kind}");
        assert!(contains_in_order(&cancelled.terminal, b":q!"));
    }
}

#[test]
fn edit_018_report_an_invalid_command() {
    for kind in ["input", "textarea"] {
        let result = super::command_line::prompt_with_delayed_keys(
            &format!("exec {{ink}} {kind} --normal --value unchanged"),
            b":nope\r",
            std::time::Duration::from_millis(1_200),
            b"\x03",
            Some("[colors]\nerror = \"#010203\"\n"),
        );
        assert_eq!(result.status, 130, "{kind}");
        assert!(
            result
                .terminal
                .windows("not a valid command".len())
                .any(|bytes| bytes == b"not a valid command"),
            "{kind}"
        );
        let error = super::sequence_position(&result.terminal, b"not a valid command");
        assert!(
            result.terminal[error..]
                .windows(6)
                .any(|bytes| bytes == b"NORMAL"),
            "{kind}"
        );
        assert!(
            result
                .terminal
                .windows("38;2;1;2;3".len())
                .any(|bytes| bytes == b"38;2;1;2;3"),
            "{kind}"
        );
    }
}

#[test]
fn edit_019_confirm_submission_before_quitting() {
    for kind in ["input", "textarea"] {
        for command in ["q", "qa"] {
            let accepted = super::command_line::prompt(
                &format!("exec {{ink}} {kind} --normal --value submitted"),
                format!(":{command}\ry").as_bytes(),
            );
            assert_eq!(accepted.status, 0, "{kind} :{command}");
            assert_eq!(accepted.stdout, b"submitted\n", "{kind} :{command}");
            assert!(
                accepted
                    .terminal
                    .windows("Submit? (y/n)".len())
                    .any(|bytes| bytes == b"Submit? (y/n)"),
                "{kind} :{command}"
            );
        }

        let resumed = super::command_line::prompt(
            &format!("exec {{ink}} {kind} --normal --value resumed"),
            b":q\r\x1b:q\rn:wq\r",
        );
        assert_eq!(resumed.status, 0, "{kind}");
        assert_eq!(resumed.stdout, b"resumed\n", "{kind}");
    }
}

#[test]
fn edit_020_append_at_the_end_of_a_line() {
    let mut editor = Editor::input("e\u{301}").expect("input fixture");
    assert!(editor.append_at_line_end());
    assert_eq!(editor.mode(), Mode::Insert);
    assert_eq!(editor.cursor(), Position::new(0, 1));
    assert!(!editor.append_at_line_end());
    editor.escape();
    assert_eq!(editor.cursor(), Position::new(0, 0));
    assert!(editor.append_at_line_end());
    editor.insert("👩‍💻");
    editor.escape();
    assert_eq!(editor.text(), "e\u{301}👩‍💻");
    assert_eq!(editor.cursor(), Position::new(0, 1));

    for (kind, value, keys, expected) in [
        ("input", "onee\u{301}", "A👩‍💻\u{1b}\u{4}", "onee\u{301}👩‍💻\n"),
        (
            "textarea",
            "onee\u{301}\ntwo",
            "A!\u{1b}\u{4}",
            "onee\u{301}!\ntwo\n",
        ),
        ("input", "", "A!\u{1b}\u{4}", "!\n"),
        ("textarea", "\ntwo", "A!\u{1b}\u{4}", "!\ntwo\n"),
        ("textarea", "one\n", "jA!\u{1b}\u{4}", "one\n!\n"),
        ("textarea", "one\n\ntwo", "jA!\u{1b}\u{4}", "one\n!\ntwo\n"),
    ] {
        let result = super::command_line::prompt(
            &format!("exec {{ink}} {kind} --normal --value '{}'", value),
            keys.as_bytes(),
        );
        assert_eq!(result.status, 0, "{kind} {value:?}");
        assert_eq!(result.stdout, expected.as_bytes(), "{kind} {value:?}");
    }
}

fn contains_in_order(haystack: &[u8], needle: &[u8]) -> bool {
    let mut remaining = needle;
    for byte in haystack {
        if remaining.first() == Some(byte) {
            remaining = &remaining[1..];
            if remaining.is_empty() {
                return true;
            }
        }
    }
    false
}

fn block_editor() -> Editor {
    let mut editor = Editor::textarea("abcd\nx\npqrs");
    editor.set_cursor(Position::new(0, 1));
    editor.enter_visual_block();
    editor.move_down();
    editor.move_down();
    editor.move_right();
    editor
}
