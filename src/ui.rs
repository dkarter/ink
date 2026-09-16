//! Composable terminal presentation for input and textarea editors.

mod cursor;
mod display;
mod viewport;
mod widgets;

pub use cursor::{CursorRequest, cursor_style, mode_label};
pub use viewport::Viewport;
pub use widgets::{Input, InputState, Textarea, TextareaState};
