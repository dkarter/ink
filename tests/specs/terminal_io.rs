use std::{
    io,
    panic::{self, AssertUnwindSafe},
    sync::{Arc, Mutex},
};

use ink::terminal::{
    CancelReason, CursorShape, ExitStatus, PromptOutcome, RuntimeIo, TerminalDevice, run_prompt,
};

const ENTER_BYTES: &[u8] = b"\x1b7\x1b[?25l";
const RESTORE_BYTES: &[u8] = b"\x1b8\x1b[J\x1b[0 q\x1b[?25h";

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

    fn fail_disable_once(&self) {
        self.0.lock().unwrap().disable_failures = 1;
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
    for outcome in [
        Ok(PromptOutcome::Accepted("value".into())),
        Ok(PromptOutcome::Cancelled(CancelReason::Interrupt)),
        Err(io::Error::other("render failed")),
    ] {
        let device = Arc::new(FakeDevice::default());
        let mut io = FakeIo::interactive(Arc::clone(&device));
        let status = run_prompt(&mut io, None, |session, _| {
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
    device.fail_disable_once();
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
}

#[test]
fn io_007_restore_after_panic() {
    let device = Arc::new(FakeDevice::default());
    let mut io = FakeIo::interactive(Arc::clone(&device));

    let panic = panic::catch_unwind(AssertUnwindSafe(|| {
        run_prompt(&mut io, None, |session, _| {
            session.set_cursor_shape(CursorShape::UnderScore)?;
            panic!("prompt panic")
        });
    }));

    assert!(panic.is_err());
    assert!(device.bytes().ends_with(RESTORE_BYTES));
    assert_eq!(device.calls().last(), Some(&"disable_raw"));
}
