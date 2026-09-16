use ink::editor::{Editor, InputError, Mode, Position};

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
    assert_eq!(textarea.cursor(), Position::new(0, 0));

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
    assert_eq!(editor.unnamed_register(), "second\nthird");
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
    assert_eq!(editor.text(), "one\n");
    assert_eq!(editor.cursor(), Position::new(0, 2));
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
