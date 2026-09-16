use std::{fs, path::Path, process::Command};

use ink::{
    config::{self, CliOptions, Config, ThemeName},
    theme::{self, Color, ColorRole},
};
use tempfile::TempDir;

#[test]
fn theme_001_use_default_theme() {
    let config = Config::default();
    let settings = ink::config::Settings::resolve(&config, &CliOptions::default());
    let resolved = theme::resolve(settings.theme, &config.colors);

    assert_eq!(resolved.name, ThemeName::TokyoNight);
}
#[test]
fn theme_002_select_every_bundled_theme() {
    let mut palettes = Vec::new();
    for name in ThemeName::ALL {
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
    }
}
#[test]
fn theme_003_command_line_theme_wins() {
    let config = Config {
        theme: Some(ThemeName::Dracula),
        ..Config::default()
    };
    let cli = CliOptions {
        theme: Some(ThemeName::Nord),
        ..CliOptions::default()
    };

    let settings = ink::config::Settings::resolve(&config, &cli);
    let resolved = theme::resolve(settings.theme, &config.colors);

    assert_eq!(resolved.name, ThemeName::Nord);
    let nord = theme::resolve(ThemeName::Nord, &Default::default());
    assert_eq!(resolved.palette, nord.palette);
}
#[test]
fn theme_004_user_colors_overlay_a_base_theme() {
    let base = theme::resolve(ThemeName::GruvboxDark, &Default::default());
    let config = config::parse(
        Path::new("config.toml"),
        "theme = \"gruvbox-dark\"\n[colors]\naccent = \"#123456\"\nvisual-mode = \"#abcdef\"\n",
    )
    .expect("parse overrides");

    let settings = ink::config::Settings::resolve(&config, &CliOptions::default());
    let resolved = theme::resolve(settings.theme, &config.colors);

    assert_eq!(resolved.palette.accent, Color::rgb(0x12, 0x34, 0x56));
    assert_eq!(resolved.palette.visual_mode, Color::rgb(0xab, 0xcd, 0xef));
    for role in ColorRole::ALL {
        if !matches!(role, ColorRole::Accent | ColorRole::VisualMode) {
            assert_eq!(
                resolved.palette.color(role),
                base.palette.color(role),
                "{role}"
            );
        }
    }
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
