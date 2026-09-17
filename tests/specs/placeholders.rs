use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, ExitCode},
};

use ink::{
    cli::{CliRuntime, PromptKind, PromptRuntimeOptions, ResolvedPromptOptions},
    config::{self, ThemeName},
    editor::{Editor, Mode},
    theme::{self, Color, ColorRole},
    ui::{Input, InputState, Textarea, TextareaState},
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

#[derive(Default)]
struct RuntimeProbe {
    prompts: Vec<(PromptKind, PromptRuntimeOptions)>,
}

impl CliRuntime for RuntimeProbe {
    fn write_stdout(&mut self, _: &str) {}

    fn run_prompt(
        &mut self,
        kind: PromptKind,
        prompt: PromptRuntimeOptions,
        _: ResolvedPromptOptions,
    ) -> ExitCode {
        self.prompts.push((kind, prompt));
        ExitCode::SUCCESS
    }
}

#[test]
fn ph_001_set_a_placeholder_for_either_prompt() {
    for (command, expected) in [
        ("input", PromptKind::Input),
        ("textarea", PromptKind::Textarea),
    ] {
        let mut runtime = RuntimeProbe::default();
        let args = [
            OsStr::new(command),
            OsStr::new("--placeholder"),
            OsStr::new("empty guidance"),
        ];

        let status = ink::cli::run_from(&args, &mut runtime).expect("parse placeholder option");

        assert_eq!(status, ExitCode::SUCCESS);
        assert_eq!(runtime.prompts.len(), 1);
        assert_eq!(runtime.prompts[0].0, expected);
        assert_eq!(runtime.prompts[0].1.placeholder, "empty guidance");
        assert_eq!(runtime.prompts[0].1.value, None);
    }

    let completion = Command::new(env!("CARGO_BIN_EXE_ink"))
        .args([
            "__complete_word__",
            "--shell",
            "zsh",
            "--line",
            "ink textarea --pla",
        ])
        .output()
        .expect("complete textarea placeholder option");
    assert!(completion.status.success());
    assert!(String::from_utf8_lossy(&completion.stdout).starts_with("--placeholder\t"));
}

#[test]
fn ph_002_show_the_placeholder_at_empty_startup() {
    let input = Editor::empty_input();
    let input_area = Rect::new(0, 0, 16, 1);
    let mut input_buffer = Buffer::empty(input_area);
    Input::new(&input).placeholder("type here").render(
        input_area,
        &mut input_buffer,
        &mut InputState::default(),
    );
    assert!(buffer_row(&input_buffer, 0).starts_with("type here"));

    let textarea = Editor::textarea("");
    let textarea_area = Rect::new(0, 0, 16, 4);
    let mut textarea_buffer = Buffer::empty(textarea_area);
    Textarea::new(&textarea)
        .placeholder("summary\ndetails")
        .render(
            textarea_area,
            &mut textarea_buffer,
            &mut TextareaState::default(),
        );
    assert!(buffer_row(&textarea_buffer, 0).starts_with("summary"));
    assert!(buffer_row(&textarea_buffer, 1).starts_with("details"));
    assert_eq!(input.text(), "");
    assert_eq!(textarea.text(), "");
}

#[test]
fn ph_003_initial_content_suppresses_the_placeholder() {
    let input = Editor::input("explicit").unwrap();
    let area = Rect::new(0, 0, 20, 1);
    let mut buffer = Buffer::empty(area);
    Input::new(&input)
        .placeholder("hidden")
        .render(area, &mut buffer, &mut InputState::default());
    assert!(buffer_row(&buffer, 0).starts_with("explicit"));
    assert!(!buffer_row(&buffer, 0).contains("hidden"));

    let piped = super::command_line::prompt(
        "printf 'piped seed' | {ink} textarea --placeholder hidden",
        b"\x04",
    );
    assert_eq!(piped.status, 0);
    assert_eq!(piped.stdout, b"piped seed\n");
    assert!(!contains(&piped.terminal, b"hidden"));
}

#[test]
fn ph_004_editing_operations_cannot_act_on_a_placeholder() {
    for visual in [
        Editor::enter_visual as fn(&mut Editor),
        Editor::enter_visual_line,
        Editor::enter_visual_block,
    ] {
        let mut yanked = Editor::textarea("");
        visual(&mut yanked);
        assert_eq!(yanked.selected_text(), "");
        assert!(yanked.yank_selection());
        assert_eq!(yanked.text(), "");
        assert_eq!(yanked.unnamed_register(), "");

        let mut deleted = Editor::textarea("");
        visual(&mut deleted);
        assert!(deleted.delete_selection());
        assert_eq!(deleted.text(), "");
        assert_eq!(deleted.unnamed_register(), "");

        let mut changed = Editor::textarea("");
        visual(&mut changed);
        assert!(changed.change_selection());
        assert_eq!(changed.mode(), Mode::Insert);
        assert_eq!(changed.text(), "");
        assert_eq!(changed.unnamed_register(), "");
    }

    let mut editor = Editor::textarea("");
    assert!(!editor.delete_at_cursor());

    let area = Rect::new(0, 0, 12, 3);
    let mut buffer = Buffer::empty(area);
    Textarea::new(&editor).placeholder("untouched").render(
        area,
        &mut buffer,
        &mut TextareaState::default(),
    );
    assert!(buffer_row(&buffer, 0).starts_with("untouched"));
}

#[test]
fn ph_005_accept_an_untouched_empty_prompt() {
    for (command, keys) in [
        ("input --placeholder guidance", b"\r".as_slice()),
        ("textarea --placeholder guidance", b"\x04".as_slice()),
    ] {
        let result = super::command_line::prompt(&format!("exec {{ink}} {command}"), keys);
        assert_eq!(result.status, 0);
        assert_eq!(result.stdout, b"\n");
        assert!(!contains(&result.stdout, b"guidance"));
        assert!(!result.stdout.contains(&0x1b));
    }
}

#[test]
fn ph_006_hide_and_restore_the_placeholder_during_editing() {
    let mut editor = Editor::empty_input();
    editor.enter_insert();
    let area = Rect::new(0, 0, 12, 1);
    let mut state = InputState::default();

    assert_eq!(
        render_input(&editor, "guidance", area, &mut state),
        "guidance    "
    );
    assert!(editor.insert("x"));
    assert!(render_input(&editor, "guidance", area, &mut state).starts_with('x'));
    assert!(editor.backspace());
    assert_eq!(editor.text(), "");
    assert_eq!(
        render_input(&editor, "guidance", area, &mut state),
        "guidance    "
    );
}

#[test]
fn ph_007_normalize_an_input_placeholder_to_one_line() {
    let result = super::command_line::prompt(
        "exec {ink} input --placeholder \"$(printf 'a\\r\\nb\\rc\\nd')\"",
        b"\x03",
    );

    assert_eq!(result.status, 130);
    assert!(contains(&result.terminal, b"abcd"));
    assert!(!contains(&result.terminal, b"a\r\nb"));
}

#[test]
fn ph_008_clip_a_multiline_textarea_placeholder_safely() {
    let editor = Editor::textarea("");
    let original_cursor = editor.cursor();
    let placeholder = "e\u{301}界x\nsecond line\nthird";
    let mut state = TextareaState::default();

    for area in [
        Rect::new(2, 3, 3, 3),
        Rect::new(2, 3, 1, 1),
        Rect::new(2, 3, 0, 3),
        Rect::new(2, 3, 3, 0),
        Rect::new(2, 3, 0, 0),
    ] {
        let mut buffer = Buffer::empty(area);
        Textarea::new(&editor)
            .placeholder(placeholder)
            .render(area, &mut buffer, &mut state);
        if let Some(cursor) = state.cursor() {
            assert!(area.contains(cursor.position));
        } else {
            assert!(area.is_empty());
        }
        assert_eq!(editor.text(), "");
        assert_eq!(editor.cursor(), original_cursor);
    }

    let area = Rect::new(0, 0, 3, 3);
    let mut buffer = Buffer::empty(area);
    Textarea::new(&editor)
        .placeholder(placeholder)
        .render(area, &mut buffer, &mut state);
    assert_eq!(buffer.cell((0, 0)).unwrap().symbol(), "e\u{301}");
    assert_eq!(buffer.cell((1, 0)).unwrap().symbol(), "界");
    assert!(buffer_row(&buffer, 1).starts_with("sec"));
    assert!(!buffer_row(&buffer, 2).contains("third"));
}

#[test]
fn ph_009_every_bundled_theme_supplies_an_accessible_placeholder_color() {
    for name in ThemeName::ALL {
        let palette = theme::resolve(name, &Default::default()).palette;
        let ratio = contrast_ratio(palette.placeholder, palette.background);
        assert_ne!(palette.placeholder, palette.background, "{name}");
        assert!(ratio >= 4.5, "{name} placeholder contrast {ratio:.2}:1");
    }
}

#[test]
fn ph_010_override_the_placeholder_color() {
    let base = theme::resolve(ThemeName::TokyoNight, &Default::default()).palette;
    let config = config::parse(
        Path::new("config.toml"),
        "[colors]\nplaceholder = \"#abcdef\"\n",
    )
    .expect("parse placeholder override");
    let resolved = theme::resolve(ThemeName::TokyoNight, &config.colors).palette;

    assert_eq!(resolved.placeholder, Color::rgb(0xab, 0xcd, 0xef));
    for role in ColorRole::ALL {
        if role != ColorRole::Placeholder {
            assert_eq!(resolved.color(role), base.color(role), "{role}");
        }
    }

    let result = super::command_line::prompt_with_config(
        "exec {ink} input --placeholder visible",
        b"\x03",
        Some("[colors]\nplaceholder = \"#abcdef\"\n"),
    );
    assert!(contains(&result.terminal, b"38;2;171;205;239"));
}

fn render_input(editor: &Editor, placeholder: &str, area: Rect, state: &mut InputState) -> String {
    let mut buffer = Buffer::empty(area);
    Input::new(editor)
        .placeholder(placeholder)
        .show_mode(false)
        .render(area, &mut buffer, state);
    buffer_row(&buffer, area.y)
}

fn buffer_row(buffer: &Buffer, y: u16) -> String {
    (buffer.area.x..buffer.area.right())
        .map(|x| buffer.cell((x, y)).unwrap().symbol())
        .collect()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|bytes| bytes == needle)
}

fn contrast_ratio(foreground: Color, background: Color) -> f64 {
    fn luminance(color: Color) -> f64 {
        fn linear(channel: u8) -> f64 {
            let channel = f64::from(channel) / 255.0;
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        }

        0.2126 * linear(color.red) + 0.7152 * linear(color.green) + 0.0722 * linear(color.blue)
    }

    let foreground = luminance(foreground);
    let background = luminance(background);
    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
}
