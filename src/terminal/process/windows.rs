#![cfg(windows)]

use std::{fs::OpenOptions, io, sync::Arc};

use crossterm_winapi::{ConsoleMode as WinConsoleMode, Handle, HandleType};

use super::{
    ProcessTerminal, RawMode,
    raw_mode::{ConsoleMode, HandleRawMode},
};
use crate::terminal::TerminalDevice;

pub(super) fn open_terminal() -> io::Result<Arc<dyn TerminalDevice>> {
    let console = WinConsoleMode::from(Handle::new(HandleType::CurrentInputHandle)?);
    Ok(Arc::new(ProcessTerminal {
        input: OpenOptions::new().read(true).open("CONIN$")?.into(),
        output: OpenOptions::new().write(true).open("CONOUT$")?.into(),
        raw_mode: HandleRawMode::new(AcquiredConsoleMode(console)),
    }))
}

struct AcquiredConsoleMode(WinConsoleMode);

impl ConsoleMode for AcquiredConsoleMode {
    fn mode(&self) -> io::Result<u32> {
        self.0.mode()
    }

    fn set_mode(&self, mode: u32) -> io::Result<()> {
        self.0.set_mode(mode)
    }
}

impl<M: ConsoleMode> RawMode for HandleRawMode<M> {
    fn enable(&self) -> io::Result<()> {
        self.enable()
    }

    fn disable(&self) -> io::Result<()> {
        self.disable()
    }
}
