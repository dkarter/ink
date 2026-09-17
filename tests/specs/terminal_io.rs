use std::{
    io::{self, Read},
    panic::{self, AssertUnwindSafe},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use ink::terminal::{
    CancelReason, ConsoleMode, CursorShape, ExitStatus, HandleRawMode, PromptOutcome, RuntimeIo,
    TerminalDevice, run_prompt,
};
use ink::{
    cli::{CliRuntime, PromptKind, PromptRuntimeOptions, ResolvedPromptOptions},
    terminal,
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

const ENTER_BYTES: &[u8] = b"\x1b[?25l\x1b[?2004h";
const RESTORE_BYTES: &[u8] = b"\x1b[1;1H\x1b[2K\x1b8\x1b[0 q\x1b[?2004l\x1b[?25h";
const FULLSCREEN_RESTORE_BYTES: &[u8] = b"\x1b[?1049l\x1b[0 q\x1b[?2004l\x1b[?25h";

#[derive(Default)]
struct DeviceState {
    input: Vec<u8>,
    bytes: Vec<u8>,
    calls: Vec<&'static str>,
    disable_failures: usize,
}

#[derive(Default)]
struct FakeDevice(Mutex<DeviceState>);

impl FakeDevice {
    fn with_input(input: &[u8]) -> Self {
        Self(Mutex::new(DeviceState {
            input: input.to_vec(),
            ..DeviceState::default()
        }))
    }

    fn bytes(&self) -> Vec<u8> {
        self.0.lock().unwrap().bytes.clone()
    }

    fn calls(&self) -> Vec<&'static str> {
        self.0.lock().unwrap().calls.clone()
    }

    fn fail_disable(&self, attempts: usize) {
        self.0.lock().unwrap().disable_failures = attempts;
    }
}

impl TerminalDevice for FakeDevice {
    fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut state = self.0.lock().unwrap();
        let count = buffer.len().min(state.input.len());
        buffer[..count].copy_from_slice(&state.input[..count]);
        state.input.drain(..count);
        state.calls.push("read_terminal");
        Ok(count)
    }

    fn write_all(&self, bytes: &[u8]) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.bytes.extend_from_slice(bytes);
        state.calls.push("write_terminal");
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        self.0.lock().unwrap().calls.push("flush_terminal");
        Ok(())
    }

    fn enable_raw_mode(&self) -> io::Result<()> {
        self.0.lock().unwrap().calls.push("enable_raw");
        Ok(())
    }

    fn disable_raw_mode(&self) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.calls.push("disable_raw");
        if state.disable_failures > 0 {
            state.disable_failures -= 1;
            Err(io::Error::other("raw restoration failed"))
        } else {
            Ok(())
        }
    }
}

struct FakeIo {
    stdin_is_terminal: bool,
    stdin: String,
    terminal: Option<Arc<FakeDevice>>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    calls: Vec<&'static str>,
}

impl FakeIo {
    fn interactive(device: Arc<FakeDevice>) -> Self {
        Self {
            stdin_is_terminal: true,
            stdin: String::new(),
            terminal: Some(device),
            stdout: Vec::new(),
            stderr: Vec::new(),
            calls: Vec::new(),
        }
    }
}

impl RuntimeIo for FakeIo {
    fn stdin_is_terminal(&self) -> bool {
        self.stdin_is_terminal
    }

    fn read_stdin(&mut self) -> io::Result<String> {
        self.calls.push("read_stdin");
        Ok(self.stdin.clone())
    }

    fn open_controlling_terminal(&mut self) -> io::Result<Arc<dyn TerminalDevice>> {
        self.calls.push("open_terminal");
        self.terminal
            .clone()
            .map(|terminal| terminal as Arc<dyn TerminalDevice>)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no tty"))
    }

    fn write_stdout(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.calls.push("write_stdout");
        self.stdout.extend_from_slice(bytes);
        Ok(())
    }

    fn write_stderr(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.calls.push("write_stderr");
        self.stderr.extend_from_slice(bytes);
        Ok(())
    }
}

#[test]
fn io_001_accepted_output_is_clean() {
    let device = Arc::new(FakeDevice::default());
    let mut io = FakeIo::interactive(Arc::clone(&device));

    let status = run_prompt(&mut io, None, |session, _| {
        session.write_ui(b"\x1b[31mprompt\x1b[0m")?;
        Ok(PromptOutcome::Accepted("accepted value".into()))
    });

    assert_eq!(status, ExitStatus::Accepted);
    assert_eq!(io.stdout, b"accepted value\n");
    assert!(!io.stdout.contains(&0x1b));
    assert!(device.bytes().contains(&0x1b));
}

#[test]
fn io_002_piped_input_retains_terminal_control() {
    let device = Arc::new(FakeDevice::with_input(b"x"));
    let mut io = FakeIo::interactive(Arc::clone(&device));
    io.stdin_is_terminal = false;
    io.stdin = "piped seed".into();

    let status = run_prompt(&mut io, None, |session, seed| {
        assert_eq!(seed, "piped seed");
        let mut input = [0];
        session.read_input(&mut input)?;
        assert_eq!(input, *b"x");
        session.write_ui(b"ui")?;
        Ok(PromptOutcome::Accepted(seed.into()))
    });

    assert_eq!(status, ExitStatus::Accepted);
    assert_eq!(io.calls[..2], ["open_terminal", "read_stdin"]);
    assert_eq!(io.stdout, b"piped seed\n");
    assert!(device.calls().contains(&"read_terminal"));
    assert!(device.bytes().windows(2).any(|bytes| bytes == b"ui"));
}

#[test]
fn io_003_cancel_without_output() {
    for reason in [CancelReason::Interrupt, CancelReason::NormalQuit] {
        let device = Arc::new(FakeDevice::default());
        let mut io = FakeIo::interactive(device);
        let status = run_prompt(&mut io, None, |_, _| Ok(PromptOutcome::Cancelled(reason)));

        assert_eq!(status, ExitStatus::Cancelled);
        assert!(io.stdout.is_empty());
    }
}

#[test]
fn io_004_exit_statuses_identify_outcomes() {
    assert_eq!(ExitStatus::Accepted.code(), 0);
    assert_eq!(ExitStatus::UsageError.code(), 2);
    assert_eq!(ExitStatus::RuntimeFailure.code(), 1);
    assert_eq!(ExitStatus::Cancelled.code(), 130);

    struct UnreachedRuntime;

    impl CliRuntime for UnreachedRuntime {
        fn write_stdout(&mut self, _: &str) {
            unreachable!()
        }

        fn run_prompt(
            &mut self,
            _: PromptKind,
            _: PromptRuntimeOptions,
            _: ResolvedPromptOptions,
        ) -> std::process::ExitCode {
            unreachable!()
        }
    }

    let args = [
        std::ffi::OsStr::new("input"),
        std::ffi::OsStr::new("--theme"),
        std::ffi::OsStr::new("not-a-theme"),
    ];
    let status = ink::cli::run_from(&args, &mut UnreachedRuntime).unwrap();
    assert_eq!(status, terminal::ExitStatus::UsageError.into());
}

#[test]
fn io_005_fail_clearly_without_a_tty() {
    let mut io = FakeIo {
        stdin_is_terminal: false,
        stdin: String::new(),
        terminal: None,
        stdout: Vec::new(),
        stderr: Vec::new(),
        calls: Vec::new(),
    };

    let status = run_prompt(&mut io, None, |_, _| unreachable!());

    assert_eq!(status, ExitStatus::RuntimeFailure);
    assert!(io.stdout.is_empty());
    assert_eq!(io.stderr, b"ink: no controlling terminal available\n");
    assert_eq!(io.calls, ["open_terminal", "write_stderr"]);
}

#[test]
fn io_006_restore_after_every_ordinary_outcome() {
    if std::env::var_os("INK_FULLSCREEN_ERROR_CHILD").is_some() {
        let mut io = ink::terminal::ProcessIo;
        let status = run_prompt(&mut io, None, |session, _| {
            session.enter_fullscreen()?;
            session.set_cursor_shape(CursorShape::Bar)?;
            Err(io::Error::other("fullscreen PTY error"))
        });
        assert_eq!(status, ExitStatus::RuntimeFailure);
        return;
    }

    for outcome in [
        Ok(PromptOutcome::Accepted("value".into())),
        Ok(PromptOutcome::Cancelled(CancelReason::Interrupt)),
        Err(io::Error::other("render failed")),
    ] {
        let device = Arc::new(FakeDevice::default());
        let mut io = FakeIo::interactive(Arc::clone(&device));
        let status = run_prompt(&mut io, None, |session, _| {
            session.enter_inline_screen(0, 1)?;
            session.set_cursor_shape(CursorShape::Bar)?;
            outcome
        });

        assert!(matches!(
            status,
            ExitStatus::Accepted | ExitStatus::Cancelled | ExitStatus::RuntimeFailure
        ));
        let bytes = device.bytes();
        assert!(bytes.starts_with(ENTER_BYTES));
        assert!(bytes.ends_with(RESTORE_BYTES));
        let calls = device.calls();
        assert_eq!(calls.first(), Some(&"enable_raw"));
        assert_eq!(calls.last(), Some(&"disable_raw"));
    }

    let device = Arc::new(FakeDevice::default());
    device.fail_disable(1);
    let mut io = FakeIo::interactive(Arc::clone(&device));
    let status = run_prompt(&mut io, None, |_, _| {
        Ok(PromptOutcome::Accepted("value".into()))
    });

    assert_eq!(status, ExitStatus::RuntimeFailure);
    assert!(io.stdout.is_empty());
    assert_eq!(
        device
            .calls()
            .iter()
            .filter(|call| **call == "disable_raw")
            .count(),
        2
    );

    struct FakeConsole {
        mode: Arc<Mutex<u32>>,
        changes: Arc<Mutex<Vec<u32>>>,
    }

    impl ConsoleMode for FakeConsole {
        fn mode(&self) -> io::Result<u32> {
            Ok(*self.mode.lock().unwrap())
        }

        fn set_mode(&self, mode: u32) -> io::Result<()> {
            *self.mode.lock().unwrap() = mode;
            self.changes.lock().unwrap().push(mode);
            Ok(())
        }
    }

    let mode = Arc::new(Mutex::new(0x01f7));
    let changes = Arc::new(Mutex::new(Vec::new()));
    let raw_mode = HandleRawMode::new(FakeConsole {
        mode: Arc::clone(&mode),
        changes: Arc::clone(&changes),
    });
    raw_mode.enable().unwrap();
    raw_mode.enable().unwrap();
    raw_mode.disable().unwrap();

    assert_eq!(*mode.lock().unwrap(), 0x01f7);
    assert_eq!(*changes.lock().unwrap(), [0x01f0, 0x01f7]);

    let device = Arc::new(FakeDevice::default());
    let mut io = FakeIo::interactive(Arc::clone(&device));
    let status = run_prompt(&mut io, None, |session, _| {
        session.enter_fullscreen()?;
        session.set_cursor_shape(CursorShape::Bar)?;
        Ok(PromptOutcome::Accepted("value".into()))
    });
    assert_eq!(status, ExitStatus::Accepted);
    assert!(
        device
            .bytes()
            .windows(8)
            .any(|bytes| bytes == b"\x1b[?1049h")
    );
    assert!(device.bytes().ends_with(FULLSCREEN_RESTORE_BYTES));

    for keys in [b"\x04".as_slice(), b"\x03".as_slice()] {
        let result = super::command_line::prompt("exec {ink} textarea --fullscreen", keys);
        let entered = super::sequence_position(&result.terminal, b"\x1b[?1049h");
        let left = super::sequence_position(&result.terminal, b"\x1b[?1049l");
        assert!(entered < left);
    }

    let terminal = pty_fixture_output(
        "specs::terminal_io::io_006_restore_after_every_ordinary_outcome",
        "INK_FULLSCREEN_ERROR_CHILD",
    );
    let entered = super::sequence_position(&terminal, b"\x1b[?1049h");
    let left = super::sequence_position(&terminal, b"\x1b[?1049l");
    let reported = super::sequence_position(&terminal, b"fullscreen PTY error");
    assert!(entered < left);
    assert!(left < reported);
}

#[test]
fn io_007_restore_after_panic() {
    if std::env::var_os("INK_FULLSCREEN_PANIC_CHILD").is_some() {
        let mut io = ink::terminal::ProcessIo;
        run_prompt(&mut io, None, |session, _| -> io::Result<PromptOutcome> {
            session.enter_fullscreen()?;
            session.set_cursor_shape(CursorShape::UnderScore)?;
            panic!("fullscreen PTY panic")
        });
        unreachable!("panic fixture must not return");
    }
    if std::env::var_os("INK_COMPACT_PANIC_CHILD").is_some() {
        let mut io = ink::terminal::ProcessIo;
        run_prompt(&mut io, None, |session, _| -> io::Result<PromptOutcome> {
            let (_, height) = crossterm::terminal::size()?;
            session.enter_inline_screen(height.saturating_sub(1), 1)?;
            session.set_cursor_shape(CursorShape::UnderScore)?;
            panic!("compact PTY panic")
        });
        unreachable!("panic fixture must not return");
    }

    let device = Arc::new(FakeDevice::default());
    let mut io = FakeIo::interactive(Arc::clone(&device));

    let panic = panic::catch_unwind(AssertUnwindSafe(|| {
        run_prompt(&mut io, None, |session, _| {
            session.enter_inline_screen(0, 1)?;
            session.set_cursor_shape(CursorShape::UnderScore)?;
            panic!("prompt panic")
        });
    }));

    assert!(panic.is_err());
    assert!(device.bytes().ends_with(RESTORE_BYTES));
    assert_eq!(device.calls().last(), Some(&"disable_raw"));

    let fullscreen = Arc::new(FakeDevice::default());
    let mut io = FakeIo::interactive(Arc::clone(&fullscreen));
    let panic = panic::catch_unwind(AssertUnwindSafe(|| {
        run_prompt(&mut io, None, |session, _| -> io::Result<PromptOutcome> {
            session.enter_fullscreen()?;
            session.set_cursor_shape(CursorShape::UnderScore)?;
            panic!("fullscreen prompt panic")
        });
    }));
    assert!(panic.is_err());
    let bytes = fullscreen.bytes();
    let entered = bytes
        .windows(8)
        .position(|bytes| bytes == b"\x1b[?1049h")
        .expect("enter alternate screen");
    let left = bytes
        .windows(8)
        .position(|bytes| bytes == b"\x1b[?1049l")
        .expect("leave alternate screen");
    assert!(left > entered);
    assert!(bytes.ends_with(FULLSCREEN_RESTORE_BYTES));

    let device = Arc::new(FakeDevice::default());
    let mut io = FakeIo::interactive(Arc::clone(&device));
    let status = run_prompt(&mut io, None, |session, _| {
        session.enter_inline_screen(0, 1)?;
        session.set_cursor_shape(CursorShape::Bar)?;
        assert!(
            std::thread::spawn(|| panic!("worker panic"))
                .join()
                .is_err()
        );
        assert!(!device.bytes().ends_with(RESTORE_BYTES));
        Ok(PromptOutcome::Cancelled(CancelReason::Interrupt))
    });

    assert_eq!(status, ExitStatus::Cancelled);
    assert!(device.bytes().ends_with(RESTORE_BYTES));

    let device = Arc::new(FakeDevice::default());
    device.fail_disable(2);
    let mut io = FakeIo::interactive(Arc::clone(&device));
    let panic = panic::catch_unwind(AssertUnwindSafe(|| {
        run_prompt(&mut io, None, |_, _| -> io::Result<PromptOutcome> {
            panic!("original prompt panic")
        });
    }))
    .expect_err("the original prompt panic must be resumed");

    assert_eq!(panic.downcast_ref::<&str>(), Some(&"original prompt panic"));
    assert!(io.stderr.is_empty());
    assert!(io.stdout.is_empty());
    assert_eq!(
        device
            .calls()
            .iter()
            .filter(|call| **call == "disable_raw")
            .count(),
        3
    );

    type Hook = Arc<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync>;

    let device = Arc::new(FakeDevice::default());
    let mut io = FakeIo::interactive(device);
    let later_hook_calls = Arc::new(AtomicUsize::new(0));
    let saved_dispatcher: Arc<Mutex<Option<Hook>>> = Arc::new(Mutex::new(None));
    let status = run_prompt(&mut io, None, |_, _| {
        let dispatcher: Hook = Arc::from(panic::take_hook());
        *saved_dispatcher.lock().unwrap() = Some(Arc::clone(&dispatcher));
        let calls = Arc::clone(&later_hook_calls);
        panic::set_hook(Box::new(move |information| {
            calls.fetch_add(1, Ordering::SeqCst);
            dispatcher(information);
        }));
        Ok(PromptOutcome::Cancelled(CancelReason::Interrupt))
    });
    let reported = panic::catch_unwind(|| panic!("after session teardown"));
    let _later_hook = panic::take_hook();
    let dispatcher = saved_dispatcher.lock().unwrap().take().unwrap();
    panic::set_hook(Box::new(move |information| dispatcher(information)));

    assert_eq!(status, ExitStatus::Cancelled);
    assert!(reported.is_err());
    assert_eq!(later_hook_calls.load(Ordering::SeqCst), 1);

    let terminal = pty_fixture_output(
        "specs::terminal_io::io_007_restore_after_panic",
        "INK_FULLSCREEN_PANIC_CHILD",
    );
    let entered = super::sequence_position(&terminal, b"\x1b[?1049h");
    let left = super::sequence_position(&terminal, b"\x1b[?1049l");
    let reported = super::sequence_position(&terminal, b"fullscreen PTY panic");
    assert!(entered < left);
    assert!(
        left < reported,
        "alternate screen must close before panic output"
    );

    let compact = pty_fixture_output(
        "specs::terminal_io::io_007_restore_after_panic",
        "INK_COMPACT_PANIC_CHILD",
    );
    let cleared = super::sequence_position(&compact, b"\x1b[8;1H\x1b[2K");
    let reported = super::sequence_position(&compact, b"compact PTY panic");
    assert!(
        cleared < reported,
        "compact rows must clear before panic output"
    );
}

fn pty_fixture_output(test_name: &str, environment: &str) -> Vec<u8> {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 8,
            cols: 50,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open panic PTY");
    let mut command = CommandBuilder::new(std::env::current_exe().expect("current test binary"));
    command.arg("--exact");
    command.arg(test_name);
    command.arg("--nocapture");
    command.env(environment, "1");
    let mut child = pair
        .slave
        .spawn_command(command)
        .expect("spawn panic fixture");
    drop(pair.slave);
    let mut reader = pair
        .master
        .try_clone_reader()
        .expect("clone panic PTY reader");
    let output = thread::spawn(move || {
        let mut output = Vec::new();
        reader.read_to_end(&mut output).expect("read panic PTY");
        output
    });
    super::wait_for_child(child.as_mut(), "terminal fixture");
    drop(pair.master);
    output.join().expect("join panic PTY reader")
}
