//! Terminal I/O lifecycle and runtime orchestration.

use std::{
    io,
    panic::{self, AssertUnwindSafe},
    process::ExitCode,
    sync::{Arc, Mutex, MutexGuard},
};

mod process;

pub use process::ProcessIo;

const SAVE_POSITION: &[u8] = b"\x1b7";
const RESTORE_POSITION: &[u8] = b"\x1b8";
const CLEAR_OWNED_REGION: &[u8] = b"\x1b[J";
const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";
const DEFAULT_CURSOR: &[u8] = b"\x1b[0 q";
const NO_TTY_DIAGNOSTIC: &str = "ink: no controlling terminal available\n";

static PANIC_HOOK_LOCK: Mutex<()> = Mutex::new(());

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
    previous_hook: Arc<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync>,
    _hook_lock: MutexGuard<'static, ()>,
}

impl TerminalSession {
    fn start(device: Arc<dyn TerminalDevice>) -> io::Result<Self> {
        let hook_lock = PANIC_HOOK_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let previous_hook: Arc<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync> =
            Arc::from(panic::take_hook());
        let restoration = Arc::new(Restoration::new(device));
        let panic_restoration = Arc::clone(&restoration);
        let chained_hook = Arc::clone(&previous_hook);
        panic::set_hook(Box::new(move |information| {
            let _ = panic_restoration.restore();
            chained_hook(information);
        }));

        let session = Self {
            restoration,
            previous_hook,
            _hook_lock: hook_lock,
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

    fn restore(&self) -> io::Result<()> {
        self.restoration.restore()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restoration.restore();
        if !std::thread::panicking() {
            let previous_hook = Arc::clone(&self.previous_hook);
            let _ = panic::take_hook();
            panic::set_hook(Box::new(move |information| previous_hook(information)));
        }
    }
}

struct Restoration {
    device: Arc<dyn TerminalDevice>,
    state: Mutex<RestorationState>,
}

#[derive(Default)]
struct RestorationState {
    raw_mode: bool,
    screen_region: bool,
    cursor_hidden: bool,
    cursor_shape: bool,
    flush_pending: bool,
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
            state.screen_region = true;
        }
        self.device.write_all(SAVE_POSITION)?;

        {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            state.cursor_hidden = true;
        }
        self.device.write_all(HIDE_CURSOR)?;
        self.device.flush()
    }

    fn restore(&self) -> io::Result<()> {
        let (raw_mode, screen_region, cursor_hidden, cursor_shape, flush_pending) = {
            let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            if !state.raw_mode
                && !state.screen_region
                && !state.cursor_hidden
                && !state.cursor_shape
                && !state.flush_pending
            {
                return Ok(());
            }
            (
                state.raw_mode,
                state.screen_region,
                state.cursor_hidden,
                state.cursor_shape,
                state.flush_pending,
            )
        };

        let mut first_error = None;
        if screen_region || cursor_hidden || cursor_shape {
            self.state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .flush_pending = true;
        }
        if screen_region {
            let restored = self.device.write_all(RESTORE_POSITION);
            let cleared = self.device.write_all(CLEAR_OWNED_REGION);
            if restored.is_ok() && cleared.is_ok() {
                self.state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .screen_region = false;
            }
            attempt(&mut first_error, restored);
            attempt(&mut first_error, cleared);
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
        if flush_pending || screen_region || cursor_hidden || cursor_shape {
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

    let mut session = match TerminalSession::start(device) {
        Ok(session) => session,
        Err(error) => return runtime_failure(io, &format!("terminal setup failed: {error}")),
    };

    let outcome = panic::catch_unwind(AssertUnwindSafe(|| prompt(&mut session, &seed)));
    if let Err(error) = session.restore() {
        return runtime_failure(io, &format!("terminal restoration failed: {error}"));
    }
    drop(session);

    match outcome {
        Ok(Ok(PromptOutcome::Accepted(mut value))) => {
            value.push('\n');
            if io.write_stdout(value.as_bytes()).is_err() {
                ExitStatus::RuntimeFailure
            } else {
                ExitStatus::Accepted
            }
        }
        Ok(Ok(PromptOutcome::Cancelled(_))) => ExitStatus::Cancelled,
        Ok(Err(error)) => runtime_failure(io, &format!("prompt failed: {error}")),
        Err(payload) => panic::resume_unwind(payload),
    }
}

fn runtime_failure(io: &mut impl RuntimeIo, message: &str) -> ExitStatus {
    let _ = io.write_stderr(format!("ink: {message}\n").as_bytes());
    ExitStatus::RuntimeFailure
}
