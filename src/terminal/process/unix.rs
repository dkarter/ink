#![cfg(unix)]

use std::{fs::OpenOptions, io, sync::Arc};

use super::{ProcessTerminal, RawMode};
use crate::terminal::TerminalDevice;

pub(super) fn open_terminal() -> io::Result<Arc<dyn TerminalDevice>> {
    let terminal = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    Ok(Arc::new(ProcessTerminal {
        input: terminal.try_clone()?.into(),
        output: terminal.into(),
        raw_mode: CrosstermRawMode,
    }))
}

struct CrosstermRawMode;

impl RawMode for CrosstermRawMode {
    fn enable(&self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()
    }

    fn disable(&self) -> io::Result<()> {
        crossterm::terminal::disable_raw_mode()
    }
}
