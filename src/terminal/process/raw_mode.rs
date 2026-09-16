use std::{io, sync::Mutex};

// ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT
const NOT_RAW_MODE_MASK: u32 = 0x0001 | 0x0002 | 0x0004;

pub trait ConsoleMode: Send + Sync {
    fn mode(&self) -> io::Result<u32>;
    fn set_mode(&self, mode: u32) -> io::Result<()>;
}

pub struct HandleRawMode<M> {
    console: M,
    original: Mutex<Option<u32>>,
}

impl<M: ConsoleMode> HandleRawMode<M> {
    pub fn new(console: M) -> Self {
        Self {
            console,
            original: Mutex::new(None),
        }
    }

    pub fn enable(&self) -> io::Result<()> {
        if self
            .original
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_some()
        {
            return Ok(());
        }
        let mode = self.console.mode()?;
        *self
            .original
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(mode);
        self.console.set_mode(mode & !NOT_RAW_MODE_MASK)
    }

    pub fn disable(&self) -> io::Result<()> {
        let original = *self
            .original
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(mode) = original {
            self.console.set_mode(mode)?;
            *self
                .original
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
        }
        Ok(())
    }
}
