//! Process-backed streams and controlling-terminal access.

use std::{
    fs::{File, OpenOptions},
    io::{self, IsTerminal, Read, Write},
    sync::{Arc, Mutex},
};

use super::{RuntimeIo, TerminalDevice};

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
        controlling_terminal().map(|(input, output)| {
            Arc::new(ProcessTerminal {
                input: Mutex::new(input),
                output: Mutex::new(output),
            }) as _
        })
    }

    fn write_stdout(&mut self, bytes: &[u8]) -> io::Result<()> {
        io::stdout().write_all(bytes)
    }

    fn write_stderr(&mut self, bytes: &[u8]) -> io::Result<()> {
        io::stderr().write_all(bytes)
    }
}

struct ProcessTerminal {
    input: Mutex<File>,
    output: Mutex<File>,
}

impl TerminalDevice for ProcessTerminal {
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
        crossterm::terminal::enable_raw_mode()
    }

    fn disable_raw_mode(&self) -> io::Result<()> {
        crossterm::terminal::disable_raw_mode()
    }
}

#[cfg(unix)]
fn controlling_terminal() -> io::Result<(File, File)> {
    let terminal = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    Ok((terminal.try_clone()?, terminal))
}

#[cfg(windows)]
fn controlling_terminal() -> io::Result<(File, File)> {
    Ok((
        OpenOptions::new().read(true).open("CONIN$")?,
        OpenOptions::new().write(true).open("CONOUT$")?,
    ))
}
