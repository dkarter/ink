//! Built-in themes and user color overrides.

mod palettes;

use std::{collections::BTreeMap, fmt, str::FromStr};

use serde::{Deserialize, Deserializer};

use crate::config::ThemeName;

/// A terminal color represented as 8-bit red, green, and blue channels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    #[must_use]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue
        )
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value
            .strip_prefix('#')
            .filter(|hex| hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .ok_or_else(|| format!("invalid color value `{value}`; expected `#RRGGBB`"))?;
        let channel = |range| {
            u8::from_str_radix(&hex[range], 16)
                .map_err(|_| format!("invalid color value `{value}`; expected `#RRGGBB`"))
        };

        Ok(Self::rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl From<Color> for ratatui::style::Color {
    fn from(color: Color) -> Self {
        Self::Rgb(color.red, color.green, color.blue)
    }
}

/// Semantic color roles consumed by terminal presentation code.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ColorRole {
    Foreground,
    Background,
    Muted,
    Accent,
    Border,
    Selection,
    Cursor,
    InsertMode,
    NormalMode,
    VisualMode,
    Error,
    Warning,
}

impl ColorRole {
    pub const ALL: [Self; 12] = [
        Self::Foreground,
        Self::Background,
        Self::Muted,
        Self::Accent,
        Self::Border,
        Self::Selection,
        Self::Cursor,
        Self::InsertMode,
        Self::NormalMode,
        Self::VisualMode,
        Self::Error,
        Self::Warning,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Foreground => "foreground",
            Self::Background => "background",
            Self::Muted => "muted",
            Self::Accent => "accent",
            Self::Border => "border",
            Self::Selection => "selection",
            Self::Cursor => "cursor",
            Self::InsertMode => "insert-mode",
            Self::NormalMode => "normal-mode",
            Self::VisualMode => "visual-mode",
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

impl fmt::Display for ColorRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ColorRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|role| role.as_str() == value)
            .ok_or_else(|| format!("unknown color role `{value}`"))
    }
}

impl<'de> Deserialize<'de> for ColorRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// A complete set of semantic colors used by Ink's presentation layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Palette {
    pub foreground: Color,
    pub background: Color,
    pub muted: Color,
    pub accent: Color,
    pub border: Color,
    pub selection: Color,
    pub cursor: Color,
    pub insert_mode: Color,
    pub normal_mode: Color,
    pub visual_mode: Color,
    pub error: Color,
    pub warning: Color,
}

impl Palette {
    #[must_use]
    pub const fn color(self, role: ColorRole) -> Color {
        match role {
            ColorRole::Foreground => self.foreground,
            ColorRole::Background => self.background,
            ColorRole::Muted => self.muted,
            ColorRole::Accent => self.accent,
            ColorRole::Border => self.border,
            ColorRole::Selection => self.selection,
            ColorRole::Cursor => self.cursor,
            ColorRole::InsertMode => self.insert_mode,
            ColorRole::NormalMode => self.normal_mode,
            ColorRole::VisualMode => self.visual_mode,
            ColorRole::Error => self.error,
            ColorRole::Warning => self.warning,
        }
    }

    fn set(&mut self, role: ColorRole, color: Color) {
        match role {
            ColorRole::Foreground => self.foreground = color,
            ColorRole::Background => self.background = color,
            ColorRole::Muted => self.muted = color,
            ColorRole::Accent => self.accent = color,
            ColorRole::Border => self.border = color,
            ColorRole::Selection => self.selection = color,
            ColorRole::Cursor => self.cursor = color,
            ColorRole::InsertMode => self.insert_mode = color,
            ColorRole::NormalMode => self.normal_mode = color,
            ColorRole::VisualMode => self.visual_mode = color,
            ColorRole::Error => self.error = color,
            ColorRole::Warning => self.warning = color,
        }
    }
}

/// The selected bundled theme and its fully overlaid semantic palette.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedTheme {
    pub name: ThemeName,
    pub palette: Palette,
}

/// Load a bundled palette and apply validated semantic role overrides.
#[must_use]
pub fn resolve(name: ThemeName, overrides: &BTreeMap<ColorRole, Color>) -> ResolvedTheme {
    let mut palette = palettes::bundled(name);

    for (&role, &color) in overrides {
        palette.set(role, color);
    }

    ResolvedTheme { name, palette }
}
