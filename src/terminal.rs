//! Terminal I/O lifecycle and runtime orchestration.

use std::{
    io::{self, Write},
    panic::{self, AssertUnwindSafe},
    process::ExitCode,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

mod panic_hook;
mod process;

pub use process::ProcessIo;
#[doc(hidden)]
pub use process::{ConsoleMode, HandleRawMode};

const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";
const DEFAULT_CURSOR: &[u8] = b"\x1b[0 q";
const ENABLE_BRACKETED_PASTE: &[u8] = b"\x1b[?2004h";
const DISABLE_BRACKETED_PASTE: &[u8] = b"\x1b[?2004l";
const ENTER_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049h";
const LEAVE_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049l";
const NO_TTY_DIAGNOSTIC: &str = "ink: no controlling terminal available\n";
static SESSION_LOCK: Mutex<()> = Mutex::new(());

/// Stable process statuses shared by prompt orchestration and the final CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExitStatus {
    Accepted = 0,
    RuntimeFailure = 1,
    UsageError = 2,
    Cancelled = 130,
}

impl ExitStatus {
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        Self::from(status.code())
    }
}

/// The user action that stopped a prompt without accepting its value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancelReason {
    Interrupt,
    NormalQuit,
}

/// Result returned by a prompt event loop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromptOutcome {
    Accepted(String),
    Cancelled(CancelReason),
}

/// Cursor shapes available to the prompt renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorShape {
    Block,
    Bar,
    UnderScore,
}

/// Injectable controlling-terminal operations.
pub trait TerminalDevice: Send + Sync {
    fn read(&self, buffer: &mut [u8]) -> io::Result<usize>;
    fn write_all(&self, bytes: &[u8]) -> io::Result<()>;
    fn flush(&self) -> io::Result<()>;
    fn enable_raw_mode(&self) -> io::Result<()>;
    fn disable_raw_mode(&self) -> io::Result<()>;
}

/// Injectable process streams and controlling-terminal acquisition.
pub trait RuntimeIo {
    fn stdin_is_terminal(&self) -> bool;
    fn read_stdin(&mut self) -> io::Result<String>;
    fn open_controlling_terminal(&mut self) -> io::Result<Arc<dyn TerminalDevice>>;
    fn write_stdout(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn write_stderr(&mut self, bytes: &[u8]) -> io::Result<()>;
}

/// Access to interactive input and UI output during a prompt.
pub struct TerminalSession {
    restoration: Arc<Restoration>,
    _panic_registration: panic_hook::Registration,
    _session_lock: MutexGuard<'static, ()>,
}

pub(crate) struct SessionWriter<'a>(pub(crate) &'a TerminalSession);

impl Write for SessionWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write_ui(bytes)?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl TerminalSession {
    fn start(device: Arc<dyn TerminalDevice>) -> io::Result<Self> {
        let session_lock = SESSION_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let restoration = Arc::new(Restoration::new(device));
        let session = Self {
            _panic_registration: panic_hook::register(Arc::clone(&restoration)),
            restoration,
            _session_lock: session_lock,
        };
        match panic::catch_unwind(AssertUnwindSafe(|| session.restoration.enter())) {
            Ok(Ok(())) => Ok(session),
            Ok(Err(error)) => Err(error),
            Err(payload) => {
                drop(session);
                panic::resume_unwind(payload)
            }
        }
    }

    /// Read an event payload from the controlling terminal, never from piped stdin.
    pub fn read_input(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.restoration.device.read(buffer)
    }

    /// Read a decoded terminal event from the controlling terminal.
    pub fn read_event(&self) -> io::Result<crossterm::event::Event> {
        crossterm::event::read()
    }

    /// Read a decoded terminal event if one arrives before the timeout.
    pub fn poll_event(&self, timeout: Duration) -> io::Result<Option<crossterm::event::Event>> {
        if crossterm::event::poll(timeout)? {
            self.read_event().map(Some)
        } else {
            Ok(None)
        }
    }

    /// Write UI bytes to the controlling terminal, never to stdout.
    pub fn write_ui(&self, bytes: &[u8]) -> io::Result<()> {
        self.restoration.device.write_all(bytes)
    }

    pub fn flush(&self) -> io::Result<()> {
        self.restoration.device.flush()
    }

    pub fn set_cursor_shape(&self, shape: CursorShape) -> io::Result<()> {
        let bytes = match shape {
            CursorShape::Block => b"\x1b[2 q".as_slice(),
            CursorShape::Bar => b"\x1b[6 q".as_slice(),
            CursorShape::UnderScore => b"\x1b[4 q".as_slice(),
        };
        self.restoration
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cursor_shape = true;
        self.restoration.device.write_all(bytes)
    }

    pub fn enter_inline_screen(&self, height: u16) -> io::Result<()> {
        {
            let mut state = self
                .restoration
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.screen = ScreenState::Inline {
                height,
                cursor_row: 0,
            };
        }
        let reserved_lines = height.saturating_sub(1);
        let mut reserve = String::new();
        reserve.push_str("\r\n".repeat(usize::from(reserved_lines)).as_str());
        if reserved_lines > 0 {
            use std::fmt::Write as _;
            write!(reserve, "\x1b[{reserved_lines}A").expect("writing to a string cannot fail");
        }
        reserve.push('\r');
        self.write_ui(reserve.as_bytes())?;
        self.flush()
    }

    pub fn track_inline_screen(&self, height: u16) {
        let mut state = self
            .restoration
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let ScreenState::Inline {
            height: tracked, ..
        } = &mut state.screen
        {
            *tracked = (*tracked).max(height);
        }
    }

    pub(crate) fn track_inline_cursor(&self, row: u16) {
        let mut state = self
            .restoration
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let ScreenState::Inline { cursor_row, .. } = &mut state.screen {
            *cursor_row = row;
        }
    }

    pub(crate) fn abandon_inline_screen(&self) {
        let mut state = self
            .restoration
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if matches!(state.screen, ScreenState::Inline { .. }) {
            state.screen = ScreenState::None;
        }
    }

    pub fn enter_fullscreen(&self) -> io::Result<()> {
        self.restoration
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .screen = ScreenState::Alternate;
        self.write_ui(ENTER_ALTERNATE_SCREEN)?;
        self.flush()
    }

    fn restore(&self) -> io::Result<()> {
        self.restoration.restore()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restoration.restore();
    }
}

struct Restoration {
    device: Arc<dyn TerminalDevice>,
    state: Mutex<RestorationState>,
}

#[derive(Default)]
struct RestorationState {
    raw_mode: bool,
    screen: ScreenState,
    cursor_hidden: bool,
    cursor_shape: bool,
    bracketed_paste: bool,
    flush_pending: bool,
}

#[derive(Clone, Default)]
enum ScreenState {
    #[default]
    None,
    Inline {
        height: u16,
        cursor_row: u16,
    },
    Alternate,
}

impl Restoration {
    fn new(device: Arc<dyn TerminalDevice>) -> Self {
        Self {
            device,
            state: Mutex::new(RestorationState::default()),
        }
    }

    fn enter(&self) -> io::Result<()> {
        {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            state.raw_mode = true;
        }
        self.device.enable_raw_mode()?;

        {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            state.cursor_hidden = true;
        }
        self.device.write_all(HIDE_CURSOR)?;
        {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            state.bracketed_paste = true;
        }
        self.device.write_all(ENABLE_BRACKETED_PASTE)?;
        self.device.flush()
    }

    fn restore(&self) -> io::Result<()> {
        let (raw_mode, screen, cursor_hidden, cursor_shape, bracketed_paste, flush_pending) = {
            let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            if !state.raw_mode
                && matches!(&state.screen, ScreenState::None)
                && !state.cursor_hidden
                && !state.cursor_shape
                && !state.bracketed_paste
                && !state.flush_pending
            {
                return Ok(());
            }
            (
                state.raw_mode,
                state.screen.clone(),
                state.cursor_hidden,
                state.cursor_shape,
                state.bracketed_paste,
                state.flush_pending,
            )
        };

        let mut first_error = None;
        let had_screen = !matches!(&screen, ScreenState::None);
        if had_screen || cursor_hidden || cursor_shape || bracketed_paste {
            self.state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .flush_pending = true;
        }
        match screen {
            ScreenState::None => {}
            ScreenState::Inline { height, cursor_row } => {
                let mut cleanup = String::with_capacity(usize::from(height) * 12);
                cleanup.push('\r');
                if cursor_row > 0 {
                    use std::fmt::Write as _;
                    write!(cleanup, "\x1b[{cursor_row}A").expect("writing to a string cannot fail");
                }
                for row in 0..height {
                    cleanup.push_str("\x1b[2K");
                    if row + 1 < height {
                        cleanup.push_str("\x1b[1B\r");
                    }
                }
                if height > 1 {
                    use std::fmt::Write as _;
                    write!(cleanup, "\x1b[{}A", height - 1)
                        .expect("writing to a string cannot fail");
                }
                cleanup.push('\r');
                // Relative movement cannot be retried safely after a partial device write.
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .screen = ScreenState::None;
                let result = self.device.write_all(cleanup.as_bytes());
                attempt(&mut first_error, result);
            }
            ScreenState::Alternate => {
                let result = self.device.write_all(LEAVE_ALTERNATE_SCREEN);
                if result.is_ok() {
                    self.state
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .screen = ScreenState::None;
                }
                attempt(&mut first_error, result);
            }
        }
        if cursor_shape {
            let result = self.device.write_all(DEFAULT_CURSOR);
            if result.is_ok() {
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .cursor_shape = false;
            }
            attempt(&mut first_error, result);
        }
        if bracketed_paste {
            let result = self.device.write_all(DISABLE_BRACKETED_PASTE);
            if result.is_ok() {
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .bracketed_paste = false;
            }
            attempt(&mut first_error, result);
        }
        if cursor_hidden {
            let result = self.device.write_all(SHOW_CURSOR);
            if result.is_ok() {
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .cursor_hidden = false;
            }
            attempt(&mut first_error, result);
        }
        if flush_pending || had_screen || cursor_hidden || cursor_shape || bracketed_paste {
            let result = self.device.flush();
            if result.is_ok() {
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .flush_pending = false;
            }
            attempt(&mut first_error, result);
        }
        if raw_mode {
            let result = self.device.disable_raw_mode();
            if result.is_ok() {
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .raw_mode = false;
            }
            attempt(&mut first_error, result);
        }
        first_error.map_or(Ok(()), Err)
    }
}

fn attempt(first_error: &mut Option<io::Error>, result: io::Result<()>) {
    if let Err(error) = result
        && first_error.is_none()
    {
        *first_error = Some(error);
    }
}

/// Run terminal setup, a prompt callback, restoration, and process output.
///
/// An explicit seed takes precedence. Otherwise piped stdin is read once as seed data;
/// all subsequent interactive reads and UI writes go through `TerminalSession`.
pub fn run_prompt<F>(
    io: &mut impl RuntimeIo,
    explicit_seed: Option<String>,
    prompt: F,
) -> ExitStatus
where
    F: FnOnce(&mut TerminalSession, &str) -> io::Result<PromptOutcome>,
{
    let device = match io.open_controlling_terminal() {
        Ok(device) => device,
        Err(_) => {
            let _ = io.write_stderr(NO_TTY_DIAGNOSTIC.as_bytes());
            return ExitStatus::RuntimeFailure;
        }
    };
    let seed = match explicit_seed {
        Some(seed) => seed,
        None if io.stdin_is_terminal() => String::new(),
        None => match io.read_stdin() {
            Ok(seed) => seed,
            Err(error) => return runtime_failure(io, &format!("cannot read stdin: {error}")),
        },
    };

    let outcome = match run_session(io, device, |session| prompt(session, &seed)) {
        Ok(outcome) => outcome,
        Err(status) => return status,
    };

    match outcome {
        PromptOutcome::Accepted(mut value) => {
            value.push('\n');
            if io.write_stdout(value.as_bytes()).is_err() {
                ExitStatus::RuntimeFailure
            } else {
                ExitStatus::Accepted
            }
        }
        PromptOutcome::Cancelled(_) => ExitStatus::Cancelled,
    }
}

/// Run a fullscreen utility UI without reading stdin or emitting an accepted value.
pub(crate) fn run_utility<T, F>(io: &mut impl RuntimeIo, utility: F) -> Result<T, ExitStatus>
where
    F: FnOnce(&mut TerminalSession) -> io::Result<T>,
{
    let device = match io.open_controlling_terminal() {
        Ok(device) => device,
        Err(_) => {
            let _ = io.write_stderr(NO_TTY_DIAGNOSTIC.as_bytes());
            return Err(ExitStatus::RuntimeFailure);
        }
    };
    run_session(io, device, utility)
}

fn run_session<T, F>(
    io: &mut impl RuntimeIo,
    device: Arc<dyn TerminalDevice>,
    operation: F,
) -> Result<T, ExitStatus>
where
    F: FnOnce(&mut TerminalSession) -> io::Result<T>,
{
    let mut session = match TerminalSession::start(device) {
        Ok(session) => session,
        Err(error) => {
            return Err(runtime_failure(
                io,
                &format!("terminal setup failed: {error}"),
            ));
        }
    };
    let outcome = panic::catch_unwind(AssertUnwindSafe(|| operation(&mut session)));
    let restoration = session.restore();
    drop(session);
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(payload) => panic::resume_unwind(payload),
    };
    if let Err(error) = restoration {
        return Err(runtime_failure(
            io,
            &format!("terminal restoration failed: {error}"),
        ));
    }
    match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(runtime_failure(io, &format!("terminal UI failed: {error}"))),
    }
}

fn runtime_failure(io: &mut impl RuntimeIo, message: &str) -> ExitStatus {
    let _ = io.write_stderr(format!("ink: {message}\n").as_bytes());
    ExitStatus::RuntimeFailure
}
