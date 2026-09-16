//! Process-backed streams and controlling-terminal access.

use std::{
    fs::File,
    io::{self, IsTerminal, Read, Write},
    sync::{Arc, Mutex},
};

use super::{RuntimeIo, TerminalDevice};

mod raw_mode;
mod unix;
mod windows;

pub use raw_mode::{ConsoleMode, HandleRawMode};

trait RawMode: Send + Sync {
    fn enable(&self) -> io::Result<()>;
    fn disable(&self) -> io::Result<()>;
}

struct ProcessTerminal<R> {
    input: Mutex<File>,
    output: Mutex<File>,
    raw_mode: R,
}

impl<R: RawMode> TerminalDevice for ProcessTerminal<R> {
    fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.input
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .read(buffer)
    }

    fn write_all(&self, bytes: &[u8]) -> io::Result<()> {
        self.output
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .write_all(bytes)
    }

    fn flush(&self) -> io::Result<()> {
        self.output
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .flush()
    }

    fn enable_raw_mode(&self) -> io::Result<()> {
        self.raw_mode.enable()
    }

    fn disable_raw_mode(&self) -> io::Result<()> {
        self.raw_mode.disable()
    }
}

/// Process-backed I/O used by final command-line orchestration.
#[derive(Default)]
pub struct ProcessIo;

impl RuntimeIo for ProcessIo {
    fn stdin_is_terminal(&self) -> bool {
        io::stdin().is_terminal()
    }

    fn read_stdin(&mut self) -> io::Result<String> {
        let mut seed = String::new();
        io::stdin().read_to_string(&mut seed)?;
        Ok(seed)
    }

    fn open_controlling_terminal(&mut self) -> io::Result<Arc<dyn TerminalDevice>> {
        #[cfg(unix)]
        {
            unix::open_terminal()
        }
        #[cfg(windows)]
        {
            windows::open_terminal()
        }
    }

    fn write_stdout(&mut self, bytes: &[u8]) -> io::Result<()> {
        io::stdout().write_all(bytes)
    }

    fn write_stderr(&mut self, bytes: &[u8]) -> io::Result<()> {
        io::stderr().write_all(bytes)
    }
}
