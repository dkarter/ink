use std::{fs, path::Path, process::Command};

use ink::{
    config::{self, CliOptions, Config, ThemeName},
    theme::{self, Color, ColorRole},
};
use tempfile::TempDir;

const REQUIRED_THEME_NAMES: [&str; 10] = [
    "tokyo-night",
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
    "dracula",
    "gruvbox-dark",
    "nord",
    "solarized-dark",
    "solarized-light",
];
const COLOR_ROLE_NAMES: [&str; 13] = [
    "foreground",
    "background",
    "muted",
    "accent",
    "border",
    "selection",
    "selection-foreground",
    "cursor",
    "insert-mode",
    "normal-mode",
    "visual-mode",
    "error",
    "warning",
];

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

#[test]
fn theme_001_use_default_theme() {
    let config = Config::default();
    let settings = ink::config::Settings::resolve(&config, &CliOptions::default());
    let resolved = theme::resolve(settings.theme, &config.colors);

    assert_eq!(resolved.name, ThemeName::TokyoNight);

    let result = super::command_line::prompt("exec {ink} input --value text", b"\x03");
    assert!(
        terminal_has_color(&result.terminal, "38", resolved.palette.foreground),
        "{}",
        String::from_utf8_lossy(&result.terminal).escape_debug()
    );
}
#[test]
fn theme_002_select_every_bundled_theme() {
    assert_eq!(ThemeName::ALL.map(ThemeName::as_str), REQUIRED_THEME_NAMES);

    let mut palettes = Vec::new();
    for canonical_name in REQUIRED_THEME_NAMES {
        let name = canonical_name.parse::<ThemeName>().expect("required theme");
        let resolved = theme::resolve(name, &Default::default());

        assert_eq!(resolved.name, name);
        for role in ColorRole::ALL {
            let color = resolved.palette.color(role);
            assert_eq!(color.to_string().parse(), Ok(color), "{name} {role}");
        }
        assert!(
            !palettes.contains(&resolved.palette),
            "duplicate palette: {name}"
        );
        palettes.push(resolved.palette);

        assert_eq!(name.as_str().parse::<ThemeName>(), Ok(name));
        assert_eq!(name.as_str().to_ascii_uppercase().parse(), Ok(name));

        let palette = resolved.palette;
        for (combination, foreground, background) in [
            (
                "foreground/background",
                palette.foreground,
                palette.background,
            ),
            ("muted/background", palette.muted, palette.background),
            ("accent/background", palette.accent, palette.background),
            (
                "insert-mode/background",
                palette.insert_mode,
                palette.background,
            ),
            (
                "normal-mode/background",
                palette.normal_mode,
                palette.background,
            ),
            (
                "visual-mode/background",
                palette.visual_mode,
                palette.background,
            ),
            ("error/background", palette.error, palette.background),
            ("warning/background", palette.warning, palette.background),
            (
                "selection-foreground/selection",
                palette.selection_foreground,
                palette.selection,
            ),
        ] {
            let ratio = contrast_ratio(foreground, background);
            assert!(
                ratio >= 4.5,
                "{canonical_name} {combination} contrast {ratio:.2}:1 is below 4.5:1"
            );
        }
    }
}
#[test]
fn theme_003_command_line_theme_wins() {
    let config = config::parse(Path::new("config.toml"), "theme = \"dracula\"\n")
        .expect("parse configured theme");
    let cli = CliOptions {
        theme: Some("NoRd".parse().expect("parse --theme value")),
        ..CliOptions::default()
    };

    let resolved = ink::cli::resolve_prompt_options(&config, &cli).theme;

    assert_eq!(resolved.name, ThemeName::Nord);
    let nord = theme::resolve(ThemeName::Nord, &Default::default());
    assert_eq!(resolved.palette, nord.palette);

    let result = super::command_line::prompt_with_config(
        "exec {ink} input --theme nord --value text",
        b"\x03",
        Some("theme = \"dracula\"\n"),
    );
    assert!(terminal_has_color(
        &result.terminal,
        "38",
        nord.palette.foreground
    ));
    let dracula = theme::resolve(ThemeName::Dracula, &Default::default());
    assert!(!terminal_has_color(
        &result.terminal,
        "38",
        dracula.palette.foreground
    ));
}
#[test]
fn theme_004_user_colors_overlay_a_base_theme() {
    assert_eq!(ColorRole::ALL.map(ColorRole::as_str), COLOR_ROLE_NAMES);
    let base = theme::resolve(ThemeName::GruvboxDark, &Default::default());
    let config = config::parse(
        Path::new("config.toml"),
        "theme = \"gruvbox-dark\"\n[colors]\naccent = \"#123456\"\nvisual-mode = \"#abcdef\"\nselection = \"#123456\"\nselection-foreground = \"#abcdef\"\n",
    )
    .expect("parse overrides");

    let settings = ink::config::Settings::resolve(&config, &CliOptions::default());
    let resolved = theme::resolve(settings.theme, &config.colors);

    assert_eq!(resolved.palette.accent, Color::rgb(0x12, 0x34, 0x56));
    assert_eq!(resolved.palette.visual_mode, Color::rgb(0xab, 0xcd, 0xef));
    for role in ColorRole::ALL {
        if !matches!(
            role,
            ColorRole::Accent
                | ColorRole::VisualMode
                | ColorRole::Selection
                | ColorRole::SelectionForeground
        ) {
            assert_eq!(
                resolved.palette.color(role),
                base.palette.color(role),
                "{role}"
            );
        }
    }

    let all_roles = COLOR_ROLE_NAMES.iter().enumerate().fold(
        String::from("[colors]\n"),
        |mut source, (index, role)| {
            use std::fmt::Write as _;
            writeln!(source, "{role} = \"#{:06x}\"", index + 1).expect("write role fixture");
            source
        },
    );
    let config = config::parse(Path::new("config.toml"), &all_roles).expect("parse every role");
    let resolved = theme::resolve(ThemeName::TokyoNight, &config.colors);
    for (index, role) in ColorRole::ALL.into_iter().enumerate() {
        assert_eq!(
            resolved.palette.color(role),
            Color::rgb(0, 0, u8::try_from(index + 1).unwrap()),
            "{role}"
        );
    }

    let result = super::command_line::prompt_with_config(
        "exec {ink} input --normal --value text",
        b"vl\x03",
        Some("[colors]\nselection = \"#123456\"\nselection-foreground = \"#abcdef\"\n"),
    );
    assert!(terminal_has_color(
        &result.terminal,
        "48",
        Color::rgb(0x12, 0x34, 0x56)
    ));
    assert!(terminal_has_color(
        &result.terminal,
        "38",
        Color::rgb(0xab, 0xcd, 0xef)
    ));
}
#[test]
fn theme_005_reject_unknown_theme_values() {
    let theme_error = "not-a-theme"
        .parse::<ThemeName>()
        .expect_err("reject unknown theme");
    assert!(theme_error.contains("not-a-theme"));

    for (source, invalid) in [
        ("theme = \"made-up\"\n", "made-up"),
        ("[colors]\nglow = \"#123456\"\n", "glow"),
        ("[colors]\naccent = \"ultraviolet\"\n", "ultraviolet"),
        ("[colors]\naccent = \"#aéaaa\"\n", "#aéaaa"),
    ] {
        let error = config::parse(Path::new("config.toml"), source)
            .expect_err("reject invalid theme configuration")
            .to_string();

        assert!(error.contains(invalid), "{error:?}");

        let temp = TempDir::new().expect("create test directory");
        let config_dir = temp.path().join("ink");
        fs::create_dir(&config_dir).expect("create config directory");
        fs::write(config_dir.join("config.toml"), source).expect("write invalid configuration");
        let output = Command::new(env!("CARGO_BIN_EXE_ink"))
            .arg("input")
            .env_clear()
            .env("XDG_CONFIG_HOME", temp.path())
            .output()
            .expect("run ink");
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");

        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(stderr.contains(invalid), "{stderr:?}");
        assert!(
            !stderr.contains("prompts are not implemented"),
            "prompt runtime started before validation: {stderr:?}"
        );
    }
}

fn terminal_has_color(bytes: &[u8], channel: &str, color: Color) -> bool {
    let sequence = format!("{channel};2;{};{};{}", color.red, color.green, color.blue);
    bytes
        .windows(sequence.len())
        .any(|window| window == sequence.as_bytes())
}
