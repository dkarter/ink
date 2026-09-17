//! Configuration discovery, loading, and precedence.

use std::{
    collections::BTreeMap,
    env, fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Deserializer};

use crate::theme::{Color, ColorRole};

const CONFIG_RELATIVE_PATH: &str = "ink/config.toml";
pub const SCHEMA_URL: &str = "https://dkarter.github.io/ink/schema/ink-config.schema.json";

/// Environment-dependent base directories used to discover Ink's config file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigPaths {
    pub xdg_config_home: Option<PathBuf>,
    pub home: Option<PathBuf>,
}

impl ConfigPaths {
    /// Capture config locations from the process environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            xdg_config_home: env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute()),
            home: env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
        }
    }

    /// Select one config path, preferring XDG without probing the fallback.
    #[must_use]
    pub fn config_file(&self) -> Option<PathBuf> {
        self.xdg_config_home
            .as_ref()
            .map(|path| path.join(CONFIG_RELATIVE_PATH))
            .or_else(|| {
                self.home
                    .as_ref()
                    .map(|path| path.join(".config").join(CONFIG_RELATIVE_PATH))
            })
    }
}

/// A bundled theme selection. Palette resolution belongs to the theme module.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeName {
    #[default]
    TokyoNight,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Dracula,
    GruvboxDark,
    Nord,
    SolarizedDark,
    SolarizedLight,
}

impl ThemeName {
    pub const ALL: [Self; 10] = [
        Self::TokyoNight,
        Self::CatppuccinLatte,
        Self::CatppuccinFrappe,
        Self::CatppuccinMacchiato,
        Self::CatppuccinMocha,
        Self::Dracula,
        Self::GruvboxDark,
        Self::Nord,
        Self::SolarizedDark,
        Self::SolarizedLight,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TokyoNight => "tokyo-night",
            Self::CatppuccinLatte => "catppuccin-latte",
            Self::CatppuccinFrappe => "catppuccin-frappe",
            Self::CatppuccinMacchiato => "catppuccin-macchiato",
            Self::CatppuccinMocha => "catppuccin-mocha",
            Self::Dracula => "dracula",
            Self::GruvboxDark => "gruvbox-dark",
            Self::Nord => "nord",
            Self::SolarizedDark => "solarized-dark",
            Self::SolarizedLight => "solarized-light",
        }
    }
}

impl fmt::Display for ThemeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ThemeName {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|theme| theme.as_str().eq_ignore_ascii_case(value))
            .ok_or_else(|| format!("unknown theme `{value}` for setting `theme`"))
    }
}

impl<'de> Deserialize<'de> for ThemeName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// User-configurable defaults loaded from TOML.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub normal: Option<bool>,
    pub theme: Option<ThemeName>,
    #[serde(default)]
    pub colors: BTreeMap<ColorRole, Color>,
}

/// Explicit command-line settings. `None` means no command-line override.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CliOptions {
    pub normal: Option<bool>,
    pub theme: Option<ThemeName>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StartupMode {
    #[default]
    Insert,
    Normal,
}

/// Fully resolved settings consumed by prompt startup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub startup_mode: StartupMode,
    pub theme: ThemeName,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            startup_mode: StartupMode::Insert,
            theme: ThemeName::TokyoNight,
        }
    }
}

impl Settings {
    /// Apply built-ins, then config values, then explicit CLI values.
    #[must_use]
    pub fn resolve(config: &Config, cli: &CliOptions) -> Self {
        let defaults = Self::default();
        let startup_mode = cli
            .normal
            .or(config.normal)
            .map_or(defaults.startup_mode, |normal| {
                if normal {
                    StartupMode::Normal
                } else {
                    StartupMode::Insert
                }
            });
        Self {
            startup_mode,
            theme: cli.theme.or(config.theme).unwrap_or(defaults.theme),
        }
    }
}

/// A configuration failure with its source file retained for diagnostics.
#[derive(Debug)]
pub struct ConfigError {
    path: PathBuf,
    kind: ConfigErrorKind,
}

#[derive(Debug)]
enum ConfigErrorKind {
    Read(io::Error),
    Parse(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ConfigErrorKind::Read(error) => {
                write!(formatter, "cannot read {}: {error}", self.path.display())
            }
            ConfigErrorKind::Parse(error) => {
                write!(
                    formatter,
                    "invalid configuration {}: {error}",
                    self.path.display()
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ConfigErrorKind::Read(error) => Some(error),
            ConfigErrorKind::Parse(error) => Some(error),
        }
    }
}

/// Load the selected config file. A missing file is equivalent to empty config.
pub fn load(paths: &ConfigPaths) -> Result<Config, ConfigError> {
    let Some(path) = paths.config_file() else {
        return Ok(Config::default());
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(error) => {
            return Err(ConfigError {
                path,
                kind: ConfigErrorKind::Read(error),
            });
        }
    };

    parse(&path, &source)
}

/// Parse configuration from a known path, retaining it in any diagnostic.
pub fn parse(path: &Path, source: &str) -> Result<Config, ConfigError> {
    toml::from_str(source).map_err(|error| ConfigError {
        path: path.to_owned(),
        kind: ConfigErrorKind::Parse(error),
    })
}

/// Persist a selected theme without discarding hand-written TOML content.
pub fn write_theme(paths: &ConfigPaths, theme: ThemeName) -> Result<PathBuf, ConfigWriteError> {
    let path = paths.config_file().ok_or(ConfigWriteError {
        path: None,
        message: "cannot resolve config path; set XDG_CONFIG_HOME or HOME".to_owned(),
    })?;
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(ConfigWriteError::io(path, "read", error)),
    };
    parse(&path, &source).map_err(|error| ConfigWriteError {
        path: None,
        message: error.to_string(),
    })?;

    let updated = update_theme_source(&source, theme).map_err(|error| ConfigWriteError {
        path: Some(path.clone()),
        message: format!("cannot edit config: {error}"),
    })?;
    if updated == source {
        return Ok(path);
    }
    let parent = path.parent().expect("config path has a parent");
    fs::create_dir_all(parent).map_err(|error| {
        ConfigWriteError::io(path.clone(), "create parent directory for", error)
    })?;
    let destination = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => fs::canonicalize(&path)
            .map_err(|error| ConfigWriteError::io(path.clone(), "resolve symlink for", error))?,
        Ok(_) => path.clone(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => path.clone(),
        Err(error) => return Err(ConfigWriteError::io(path.clone(), "inspect", error)),
    };
    atomic_write(&destination, updated.as_bytes())
        .map_err(|error| ConfigWriteError::io(path.clone(), "write", error))?;
    Ok(path)
}

fn update_theme_source(source: &str, theme: ThemeName) -> Result<String, toml_edit::TomlError> {
    let bytes = source.as_bytes();
    let uses_crlf = bytes.windows(2).any(|pair| pair == b"\r\n")
        && !bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| *byte == b'\n' && (index == 0 || bytes[index - 1] != b'\r'));
    let mut document = source.parse::<toml_edit::DocumentMut>()?;
    let mut value = toml_edit::Value::from(theme.as_str());
    if let Some(decor) = document
        .get("theme")
        .and_then(toml_edit::Item::as_value)
        .map(|value| value.decor().clone())
    {
        *value.decor_mut() = decor;
    }
    document["theme"] = toml_edit::Item::Value(value);
    let edited = document.to_string();
    let edited = if uses_crlf {
        edited.replace('\n', "\r\n")
    } else {
        edited
    };
    Ok(associate_schema(&edited))
}

fn associate_schema(source: &str) -> String {
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let schema = format!("#:schema {SCHEMA_URL}");
    let mut output = String::with_capacity(source.len() + schema.len() + newline.len());
    let mut replaced = false;
    for line in source.split_inclusive('\n') {
        let ending = if line.ends_with("\r\n") {
            "\r\n"
        } else if line.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        let content = line.strip_suffix(ending).unwrap_or(line);
        if !replaced && content.trim_start().starts_with("#:schema ") {
            output.push_str(&schema);
            output.push_str(ending);
            replaced = true;
        } else {
            output.push_str(line);
        }
    }
    if replaced {
        output
    } else {
        format!("{schema}{newline}{output}")
    }
}

fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = atomic_write_file::AtomicWriteFile::open(path)?;
    file.write_all(contents)?;
    file.commit()
}

/// A failure while resolving or updating Ink's configuration file.
#[derive(Debug)]
pub struct ConfigWriteError {
    path: Option<PathBuf>,
    message: String,
}

impl ConfigWriteError {
    fn io(path: PathBuf, operation: &str, error: io::Error) -> Self {
        Self {
            path: Some(path),
            message: format!("cannot {operation} config: {error}"),
        }
    }
}

impl fmt::Display for ConfigWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(formatter, "{}: {}", path.display(), self.message)
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for ConfigWriteError {}
