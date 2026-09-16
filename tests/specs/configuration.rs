use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use ink::config::{self, CliOptions, ConfigPaths, Settings, StartupMode, ThemeName};
use tempfile::TempDir;

fn write(temp: &TempDir, relative: &str, contents: &str) -> PathBuf {
    let path = temp.path().join(relative);
    fs::create_dir_all(path.parent().expect("config has parent")).expect("create config directory");
    fs::write(&path, contents).expect("write config");
    path
}

#[test]
fn cfg_001_prefer_xdg_config_home() {
    let temp = TempDir::new().expect("create test directory");
    write(&temp, "xdg/ink/config.toml", "theme = \"nord\"\n");
    write(&temp, "home/.config/ink/config.toml", "not valid TOML");

    let config = config::load(&ConfigPaths {
        xdg_config_home: Some(temp.path().join("xdg")),
        home: Some(temp.path().join("home")),
    })
    .expect("load XDG config");

    assert_eq!(config.theme, Some(ThemeName::Nord));
}
#[test]
fn cfg_002_fall_back_to_home_config() {
    let temp = TempDir::new().expect("create test directory");
    write(
        &temp,
        "home/.config/ink/config.toml",
        "normal = true\ntheme = \"solarized-light\"\n",
    );

    let config = config::load(&ConfigPaths {
        xdg_config_home: None,
        home: Some(temp.path().join("home")),
    })
    .expect("load home config");

    assert_eq!(config.normal, Some(true));
    assert_eq!(config.theme, Some(ThemeName::SolarizedLight));
}
#[test]
fn cfg_003_command_line_overrides_configuration() {
    let config = config::parse(
        Path::new("config.toml"),
        "normal = false\ntheme = \"dracula\"\n",
    )
    .expect("parse config");

    let settings = Settings::resolve(
        config,
        CliOptions {
            normal: Some(true),
            theme: Some(ThemeName::GruvboxDark),
        },
    );

    assert_eq!(settings.startup_mode, StartupMode::Normal);
    assert_eq!(settings.theme, ThemeName::GruvboxDark);
}
#[test]
fn cfg_004_invalid_configuration_is_actionable() {
    let cases = [
        ("theme = [", "theme"),
        ("theme = \"made-up\"", "theme"),
        ("normal = \"sometimes\"", "normal"),
        ("mystery = true", "mystery"),
    ];

    for (source, setting) in cases {
        let temp = TempDir::new().expect("create test directory");
        let config_path = write(&temp, "xdg/ink/config.toml", source);
        let output = Command::new(env!("CARGO_BIN_EXE_ink"))
            .arg("input")
            .env_clear()
            .env("XDG_CONFIG_HOME", temp.path().join("xdg"))
            .output()
            .expect("run ink");

        assert_eq!(output.status.code(), Some(1), "source: {source}");
        assert!(output.stdout.is_empty(), "source: {source}");
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(
            stderr.contains(&config_path.display().to_string()),
            "missing path in {stderr:?}"
        );
        assert!(
            stderr.contains(setting),
            "missing setting `{setting}` in {stderr:?}"
        );
    }
}
