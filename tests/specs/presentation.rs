use crossterm::cursor::SetCursorStyle;
use ink::{
    config::ThemeName,
    editor::{Editor, Mode, Position},
    theme,
    ui::{Input, InputState, Textarea, TextareaState},
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

#[test]
fn ui_001_cursor_shape_follows_mode() {
    for mode in [
        Mode::Insert,
        Mode::Normal,
        Mode::Visual,
        Mode::VisualLine,
        Mode::VisualBlock,
    ] {
        let editor = textarea_in_mode(mode);
        let mut state = TextareaState::default();
        let area = Rect::new(0, 0, 20, 3);
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor).render(area, &mut buffer, &mut state);
        let expected = if mode == Mode::Insert {
            SetCursorStyle::SteadyBar
        } else {
            SetCursorStyle::SteadyBlock
        };
        assert_eq!(state.cursor().unwrap().style, expected);
    }
}

#[test]
fn ui_002_mode_indicator_names_every_mode() {
    for (mode, expected) in [
        (Mode::Insert, "INSERT"),
        (Mode::Normal, "NORMAL"),
        (Mode::Visual, "VISUAL"),
        (Mode::VisualLine, "VISUAL LINE"),
        (Mode::VisualBlock, "VISUAL BLOCK"),
    ] {
        let editor = textarea_in_mode(mode);
        let mut state = TextareaState::default();
        let area = Rect::new(0, 0, 20, 3);
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor).render(area, &mut buffer, &mut state);
        assert_eq!(buffer_row(&buffer, area.bottom() - 1).trim(), expected);

        let area = Rect::new(0, 0, 20, 1);
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor).render(area, &mut buffer, &mut state);
        assert!(buffer_row(&buffer, 0).contains(expected));
        assert!(area.contains(state.cursor().unwrap().position));

        let area = Rect::new(0, 0, u16::try_from(expected.len() + 1).unwrap(), 1);
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor).render(area, &mut buffer, &mut state);
        let row = buffer_row(&buffer, 0);
        assert!(row.starts_with('t'));
        assert!(!row.contains(expected));
        assert!(area.contains(state.cursor().unwrap().position));
    }

    let editor = Editor::textarea("界");
    let mut state = TextareaState::default();
    let area = Rect::new(0, 0, 8, 1);
    let mut buffer = Buffer::empty(area);
    Textarea::new(&editor).render(area, &mut buffer, &mut state);
    assert_eq!(buffer.cell((0, 0)).unwrap().symbol(), "界");
    assert!(!buffer_row(&buffer, 0).contains("NORMAL"));

    let area = Rect::new(0, 0, 9, 1);
    let mut buffer = Buffer::empty(area);
    Textarea::new(&editor).render(area, &mut buffer, &mut state);
    assert_eq!(buffer.cell((0, 0)).unwrap().symbol(), "界");
    assert!(buffer_row(&buffer, 0).contains("NORMAL"));
}

#[test]
fn ui_003_input_remains_usable_when_narrow() {
    let mut editor = Editor::input("value").unwrap();
    editor.enter_insert();
    editor.set_cursor(Position::new(0, 5));
    let mut state = InputState::default();

    let mut wide = Buffer::empty(Rect::new(0, 0, 24, 1));
    Input::new(&editor)
        .prompt("Prompt: ")
        .render(wide.area, &mut wide, &mut state);
    assert_eq!(wide.cell((0, 0)).unwrap().symbol(), "P");
    assert!(buffer_row(&wide, 0).contains("INSERT"));

    let mut narrow = Buffer::empty(Rect::new(0, 0, 1, 1));
    Input::new(&editor)
        .prompt("Prompt: ")
        .render(narrow.area, &mut narrow, &mut state);
    let cursor = state.cursor().expect("one cell still has a cursor");
    assert_eq!(cursor.position.x, 0);
    assert_eq!(cursor.position.y, 0);
    assert_eq!(cursor.style, SetCursorStyle::SteadyBar);
    assert_eq!(narrow.cell((0, 0)).unwrap().symbol(), " ");
}

#[test]
fn ui_004_textarea_scrolls_around_cursor() {
    let original = "top\nshort\n界x\nab界cd\nbottom";
    let mut editor = Editor::textarea(original);
    editor.set_cursor(Position::new(3, 2));
    let mut state = TextareaState::default();
    let area = Rect::new(0, 0, 3, 3);
    let mut buffer = Buffer::empty(area);

    Textarea::new(&editor).render(area, &mut buffer, &mut state);

    assert_eq!(editor.text(), original);
    assert_eq!(state.viewport().top(), 2);
    assert_eq!(state.viewport().left(), 1);
    let cursor = state.cursor().unwrap();
    assert_eq!(cursor.position.x, 1);
    assert_eq!(cursor.position.y, 1);
    assert_eq!(buffer.cell(cursor.position).unwrap().symbol(), "界");
    assert!(usize::from(cursor.position.x) + 2 <= usize::from(area.width));
    assert_eq!(buffer.cell((0, 0)).unwrap().symbol(), " ");
    assert_eq!(buffer.cell((1, 0)).unwrap().symbol(), "x");

    for (text, column, expected_text, expected_x) in [
        ("\ta", 0, "a", 0),
        ("\ta", 1, "a", 0),
        ("\u{301}a", 0, "a", 0),
        ("\u{301}a", 1, "a", 0),
        ("a\tb", 2, "ab", 1),
        ("a\u{7}b", 2, "ab", 1),
    ] {
        let mut editor = Editor::textarea(text);
        editor.set_cursor(Position::new(0, column));
        let mut state = TextareaState::default();
        let area = Rect::new(0, 0, 4, 2);
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor).render(area, &mut buffer, &mut state);

        assert_eq!(buffer_row(&buffer, 0).trim_end(), expected_text);
        assert_eq!(state.cursor().unwrap().position.x, expected_x);
    }

    for (text, expected_symbol, following_x) in [
        ("\u{ff9e}b", "\u{ff9e}", 1),
        ("\u{ff9f}b", "\u{ff9f}", 1),
        ("ｶﾞb", "ｶﾞ", 2),
        ("ﾊﾟb", "ﾊﾟ", 2),
    ] {
        let mut editor = Editor::textarea(text);
        editor.set_cursor(Position::new(0, 1));
        let mut state = TextareaState::default();
        let area = Rect::new(0, 0, 4, 2);
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor).render(area, &mut buffer, &mut state);

        assert_eq!(buffer.cell((0, 0)).unwrap().symbol(), expected_symbol);
        assert_eq!(buffer.cell((following_x, 0)).unwrap().symbol(), "b");
        assert_eq!(state.cursor().unwrap().position.x, following_x);
    }
}

#[test]
fn ui_005_resize_triggers_bounded_redraw() {
    let mut editor = Editor::textarea("first\nsecond line\nthird");
    editor.set_cursor(Position::new(2, 4));
    let mut state = TextareaState::default();

    for area in [
        Rect::new(4, 2, 20, 5),
        Rect::new(4, 2, 2, 2),
        Rect::new(4, 2, 20, 0),
        Rect::new(4, 2, 0, 20),
        Rect::new(4, 2, 0, 0),
        Rect::new(4, 2, 12, 4),
    ] {
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor).render(area, &mut buffer, &mut state);
        if let Some(cursor) = state.cursor() {
            assert!(area.contains(cursor.position));
        } else {
            assert!(area.is_empty());
        }
    }

    assert_eq!(state.viewport().top(), 0);
    assert_eq!(state.viewport().left(), 0);

    let mut input = Editor::input("long input").unwrap();
    input.enter_insert();
    input.set_cursor(Position::new(0, 10));
    let mut input_state = InputState::default();
    for area in [
        Rect::new(3, 1, 20, 1),
        Rect::new(3, 1, 1, 1),
        Rect::new(3, 1, 20, 0),
        Rect::new(3, 1, 1, 0),
        Rect::new(3, 1, 0, 20),
        Rect::new(3, 1, 0, 1),
        Rect::new(3, 1, 0, 0),
        Rect::new(3, 1, 12, 1),
    ] {
        let mut buffer = Buffer::empty(area);
        Input::new(&input).render(area, &mut buffer, &mut input_state);
        if let Some(cursor) = input_state.cursor() {
            assert!(area.contains(cursor.position));
        } else {
            assert!(area.is_empty());
        }
    }
}

#[test]
fn ui_006_input_background_is_opt_in() {
    let editor = Editor::input("value").unwrap();
    let palette = theme::resolve(ThemeName::TokyoNight, &Default::default()).palette;
    let area = Rect::new(0, 0, 10, 1);

    let mut transparent = Buffer::empty(area);
    Input::new(&editor)
        .palette(palette)
        .render(area, &mut transparent, &mut InputState::default());
    assert_eq!(
        transparent.cell((0, 0)).unwrap().bg,
        ratatui::style::Color::Reset
    );

    let mut configured = Buffer::empty(area);
    Input::new(&editor)
        .palette(palette)
        .background(true)
        .render(area, &mut configured, &mut InputState::default());
    assert_eq!(
        configured.cell((0, 0)).unwrap().bg,
        palette.background.into()
    );

    let default = super::command_line::prompt("exec {ink} input --value text", b"\x03");
    let tokyo_background = format!(
        "48;2;{};{};{}",
        palette.background.red, palette.background.green, palette.background.blue
    );
    assert!(
        !default
            .terminal
            .windows(tokyo_background.len())
            .any(|bytes| bytes == tokyo_background.as_bytes())
    );

    let overridden = super::command_line::prompt_with_config(
        "exec {ink} input --value text",
        b"\x03",
        Some("[colors]\nbackground = \"#010203\"\n"),
    );
    assert!(
        overridden
            .terminal
            .windows("48;2;1;2;3".len())
            .any(|bytes| bytes == b"48;2;1;2;3")
    );
}

fn textarea_in_mode(mode: Mode) -> Editor {
    let mut editor = Editor::textarea("text");
    match mode {
        Mode::Normal => {}
        Mode::Insert => editor.enter_insert(),
        Mode::Visual => editor.enter_visual(),
        Mode::VisualLine => editor.enter_visual_line(),
        Mode::VisualBlock => editor.enter_visual_block(),
    }
    editor
}

fn buffer_row(buffer: &Buffer, y: u16) -> String {
    (buffer.area.x..buffer.area.right())
        .map(|x| buffer.cell((x, y)).unwrap().symbol())
        .collect()
}
