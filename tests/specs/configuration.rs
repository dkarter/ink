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
fn cfg_005_ignore_invalid_xdg_config_home() {
    let temp = TempDir::new().expect("create test directory");

    let home_config = write(
        &temp,
        "actual-home/.config/ink/config.toml",
        "mystery = true",
    );
    write(&temp, "relative-xdg/ink/config.toml", "theme = \"nord\"\n");
    write(&temp, "ink/config.toml", "theme = \"dracula\"\n");

    for xdg_config_home in ["relative-xdg", ""] {
        let output = Command::new(env!("CARGO_BIN_EXE_ink"))
            .arg("input")
            .current_dir(temp.path())
            .env_clear()
            .env("XDG_CONFIG_HOME", xdg_config_home)
            .env("HOME", temp.path().join("actual-home"))
            .output()
            .expect("run ink");

        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(
            stderr.contains(&home_config.display().to_string()),
            "XDG_CONFIG_HOME={xdg_config_home:?} did not fall back to HOME: {stderr:?}"
        );
        assert!(stderr.contains("mystery"));
    }
}
#[test]
fn cfg_003_command_line_overrides_configuration() {
    let config = config::parse(
        Path::new("config.toml"),
        "normal = false\ntheme = \"dracula\"\n",
    )
    .expect("parse config");

    let settings = Settings::resolve(
        &config,
        &CliOptions {
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

#[test]
fn cfg_006_validate_valid_or_absent_configuration() {
    for source in [None, Some("normal = true\ntheme = \"nord\"\n")] {
        let temp = TempDir::new().expect("create test directory");
        let config_path = temp.path().join("xdg/ink/config.toml");
        if let Some(source) = source {
            write(&temp, "xdg/ink/config.toml", source);
        }
        let output = Command::new(env!("CARGO_BIN_EXE_ink"))
            .args(["config", "validate"])
            .env_clear()
            .env("XDG_CONFIG_HOME", temp.path().join("xdg"))
            .output()
            .expect("validate config");

        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
        assert!(stdout.contains("configuration is valid"), "{stdout}");
        assert!(
            stdout.contains(&config_path.display().to_string()),
            "{stdout}"
        );
    }
}

#[test]
fn cfg_007_diagnose_invalid_or_unreadable_configuration() {
    for directory_instead_of_file in [false, true] {
        let temp = TempDir::new().expect("create test directory");
        let config_path = temp.path().join("xdg/ink/config.toml");
        fs::create_dir_all(config_path.parent().unwrap()).expect("create config parent");
        if directory_instead_of_file {
            fs::create_dir(&config_path).expect("create unreadable config path");
        } else {
            fs::write(&config_path, "unknown = true\n").expect("write invalid config");
        }
        let output = Command::new(env!("CARGO_BIN_EXE_ink"))
            .args(["config", "validate"])
            .env_clear()
            .env("XDG_CONFIG_HOME", temp.path().join("xdg"))
            .output()
            .expect("validate config");

        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(
            stderr.contains(&config_path.display().to_string()),
            "{stderr}"
        );
    }

    let usage = Command::new(env!("CARGO_BIN_EXE_ink"))
        .arg("config")
        .output()
        .expect("run invalid usage");
    assert_eq!(usage.status.code(), Some(2));
}

#[test]
fn cfg_008_describe_the_complete_config_surface() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../website/public/schema/ink-config.schema.json"
    ))
    .expect("schema is JSON");
    assert_eq!(schema["additionalProperties"], false);
    let root = schema["properties"].as_object().expect("root properties");
    assert_eq!(
        root.keys().map(String::as_str).collect::<Vec<_>>(),
        ["colors", "normal", "theme"]
    );
    assert_eq!(root["normal"]["default"], false);
    assert_eq!(root["theme"]["default"], "tokyo-night");
    assert_eq!(
        root["theme"]["anyOf"][0]["enum"],
        serde_json::json!(ThemeName::ALL.map(ThemeName::as_str))
    );
    let case_insensitive = root["theme"]["anyOf"][1]["pattern"]
        .as_str()
        .expect("case-insensitive theme pattern");
    assert!(case_insensitive.contains("[Nn][Oo][Rr][Dd]"));
    assert_eq!(root["colors"]["additionalProperties"], false);
    let colors = root["colors"]["properties"]
        .as_object()
        .expect("color properties");
    assert_eq!(
        colors
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        ink::theme::ColorRole::ALL
            .map(ink::theme::ColorRole::as_str)
            .into_iter()
            .collect()
    );
    for property in root.values().chain(colors.values()) {
        assert!(property["description"].is_string(), "{property}");
    }
    assert_eq!(schema["$defs"]["color"]["type"], "string");
    assert!(schema["$defs"]["color"]["pattern"].is_string());
}

#[test]
fn cfg_009_mutate_configuration_without_data_loss() {
    let temp = TempDir::new().expect("create test directory");
    let path = write(
        &temp,
        "xdg/ink/config.toml",
        "# personal defaults\nnormal = true\n\"theme\" = 'dracula' # keep note\n\n[colors]\naccent = \"#123456\"\n",
    );
    let paths = ConfigPaths {
        xdg_config_home: Some(temp.path().join("xdg")),
        home: None,
    };
    assert_eq!(
        config::write_theme(&paths, ThemeName::Nord).expect("write theme"),
        path
    );
    let source = fs::read_to_string(&path).expect("read updated config");
    assert!(source.starts_with(&format!("#:schema {}\n", config::SCHEMA_URL)));
    assert!(source.contains("# personal defaults"));
    assert!(source.contains("normal = true"));
    assert!(source.contains("\"theme\" = \"nord\" # keep note"));
    assert!(source.contains("[colors]\naccent = \"#123456\""));
    assert_eq!(
        config::parse(Path::new("config.toml"), &source)
            .expect("updated config is valid")
            .theme,
        Some(ThemeName::Nord)
    );

    fs::write(
        &path,
        "# Windows file\r\ntheme = \"\"\"dracula\"\"\" # multiline syntax\r\nnormal = false\r\n",
    )
    .expect("write CRLF config");
    config::write_theme(&paths, ThemeName::SolarizedLight).expect("update CRLF config");
    let source = fs::read_to_string(path).expect("read CRLF config");
    assert!(
        source.contains("theme = \"solarized-light\" # multiline syntax\r\n"),
        "{source:?}"
    );
    assert!(source.contains("normal = false\r\n"));
    assert!(
        source
            .as_bytes()
            .windows(2)
            .filter(|bytes| *bytes == b"\r\n")
            .count()
            >= 4
    );

    #[cfg(unix)]
    {
        let linked = TempDir::new().expect("create symlink test directory");
        let target = write(&linked, "target.toml", "theme = \"dracula\"\n");
        let link = linked.path().join("xdg/ink/config.toml");
        fs::create_dir_all(link.parent().unwrap()).expect("create symlink config parent");
        std::os::unix::fs::symlink(&target, &link).expect("create config symlink");
        let linked_paths = ConfigPaths {
            xdg_config_home: Some(linked.path().join("xdg")),
            home: None,
        };

        config::write_theme(&linked_paths, ThemeName::Nord).expect("update symlinked config");

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(
            fs::read_to_string(target)
                .unwrap()
                .contains("theme = \"nord\"")
        );
    }
}
