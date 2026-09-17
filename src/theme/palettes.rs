use crate::{
    config::ThemeName,
    theme::{Color, Palette},
};

const fn rgb(hex: u32) -> Color {
    Color::rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

macro_rules! palette {
    ($foreground:expr, $background:expr, $muted:expr, $accent:expr, $border:expr,
     $selection:expr, $selection_foreground:expr, $cursor:expr, $insert:expr,
     $normal:expr, $visual:expr, $error:expr, $warning:expr) => {
        Palette {
            foreground: rgb($foreground),
            background: rgb($background),
            muted: rgb($muted),
            placeholder: rgb($muted),
            accent: rgb($accent),
            border: rgb($border),
            selection: rgb($selection),
            selection_foreground: rgb($selection_foreground),
            cursor: rgb($cursor),
            insert_mode: rgb($insert),
            normal_mode: rgb($normal),
            visual_mode: rgb($visual),
            error: rgb($error),
            warning: rgb($warning),
        }
    };
}

const TOKYO_NIGHT: Palette = palette!(
    0xc0caf5, 0x1a1b26, 0x7f89b5, 0x7aa2f7, 0x3b4261, 0x33467c, 0xc0caf5, 0xc0caf5, 0x9ece6a,
    0x7aa2f7, 0xbb9af7, 0xf7768e, 0xe0af68
);
const CATPPUCCIN_LATTE: Palette = palette!(
    0x4c4f69, 0xeff1f5, 0x62657a, 0x185abd, 0x9ca0b0, 0xacb0be, 0x303247, 0x4c4f69, 0x287a1e,
    0x185abd, 0x8839ef, 0xd20f39, 0x8a5700
);
const CATPPUCCIN_FRAPPE: Palette = palette!(
    0xc6d0f5, 0x303446, 0x949cbb, 0x8caaee, 0x626880, 0x51576d, 0xc6d0f5, 0xf2d5cf, 0xa6d189,
    0x8caaee, 0xca9ee6, 0xe78284, 0xe5c890
);
const CATPPUCCIN_MACCHIATO: Palette = palette!(
    0xcad3f5, 0x24273a, 0x939ab7, 0x8aadf4, 0x5b6078, 0x494d64, 0xcad3f5, 0xf4dbd6, 0xa6da95,
    0x8aadf4, 0xc6a0f6, 0xed8796, 0xeed49f
);
const CATPPUCCIN_MOCHA: Palette = palette!(
    0xcdd6f4, 0x1e1e2e, 0x858aa3, 0x89b4fa, 0x585b70, 0x45475a, 0xcdd6f4, 0xf5e0dc, 0xa6e3a1,
    0x89b4fa, 0xcba6f7, 0xf38ba8, 0xf9e2af
);
const DRACULA: Palette = palette!(
    0xf8f8f2, 0x282a36, 0x8292c4, 0x8be9fd, 0x44475a, 0x44475a, 0xf8f8f2, 0xf8f8f2, 0x50fa7b,
    0x8be9fd, 0xbd93f9, 0xff5555, 0xf1fa8c
);
const GRUVBOX_DARK: Palette = palette!(
    0xebdbb2, 0x282828, 0xa89984, 0x83a598, 0x665c54, 0x504945, 0xebdbb2, 0xebdbb2, 0xb8bb26,
    0x83a598, 0xd3869b, 0xff5f4f, 0xfabd2f
);
const NORD: Palette = palette!(
    0xd8dee9, 0x2e3440, 0x909db6, 0x88c0d0, 0x4c566a, 0x434c5e, 0xd8dee9, 0xe5e9f0, 0xa3be8c,
    0x88c0d0, 0xc19ac7, 0xe1848c, 0xebcb8b
);
const SOLARIZED_DARK: Palette = palette!(
    0x93a1a1, 0x002b36, 0x839496, 0x4aa3d8, 0x586e75, 0x073642, 0x93a1a1, 0x93a1a1, 0x9bad00,
    0x4aa3d8, 0x9296e8, 0xff5f52, 0xd5a700
);
const SOLARIZED_LIGHT: Palette = palette!(
    0x586e75, 0xfdf6e3, 0x586e75, 0x006da8, 0x93a1a1, 0xeee8d5, 0x3f535a, 0x586e75, 0x557500,
    0x006da8, 0x514fb2, 0xc51b18, 0x7a5d00
);

pub(super) const fn bundled(name: ThemeName) -> Palette {
    match name {
        ThemeName::TokyoNight => TOKYO_NIGHT,
        ThemeName::CatppuccinLatte => CATPPUCCIN_LATTE,
        ThemeName::CatppuccinFrappe => CATPPUCCIN_FRAPPE,
        ThemeName::CatppuccinMacchiato => CATPPUCCIN_MACCHIATO,
        ThemeName::CatppuccinMocha => CATPPUCCIN_MOCHA,
        ThemeName::Dracula => DRACULA,
        ThemeName::GruvboxDark => GRUVBOX_DARK,
        ThemeName::Nord => NORD,
        ThemeName::SolarizedDark => SOLARIZED_DARK,
        ThemeName::SolarizedLight => SOLARIZED_LIGHT,
    }
}
