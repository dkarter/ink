use std::{
    fs,
    io::{Read, Write},
    sync::mpsc,
    thread,
    time::Duration,
};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tempfile::TempDir;

pub(super) struct PromptResult {
    pub(super) status: u32,
    pub(super) stdout: Vec<u8>,
    pub(super) terminal: Vec<u8>,
}

pub(super) fn prompt(command: &str, keys: &[u8]) -> PromptResult {
    prompt_with_config(command, keys, None)
}

pub(super) fn prompt_with_config(
    command: &str,
    keys: &[u8],
    config_source: Option<&str>,
) -> PromptResult {
    let temp = TempDir::new().expect("create prompt test directory");
    let stdout = temp.path().join("stdout");
    let config = temp.path().join("config");
    fs::create_dir(&config).expect("create empty config directory");
    if let Some(source) = config_source {
        let ink_config = config.join("ink");
        fs::create_dir(&ink_config).expect("create Ink config directory");
        fs::write(ink_config.join("config.toml"), source).expect("write Ink config");
    }
    let binary = env!("CARGO_BIN_EXE_ink");
    let command = command.replace("{ink}", &shell_word(binary));
    let script = format!(
        "export XDG_CONFIG_HOME={} HOME={}; {command} > {}",
        shell_word(&config.display().to_string()),
        shell_word(&temp.path().display().to_string()),
        shell_word(&stdout.display().to_string()),
    );

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 8,
            cols: 50,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open PTY");
    let mut shell = CommandBuilder::new("/bin/sh");
    shell.arg("-c");
    shell.arg(script);
    let mut child = pair.slave.spawn_command(shell).expect("spawn prompt");
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().expect("clone PTY reader");
    let (sender, receiver) = mpsc::channel();
    let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut output = Vec::new();
        let mut ready = false;
        let mut buffer = [0; 4096];
        loop {
            let count = reader.read(&mut buffer).expect("read PTY output");
            if count == 0 {
                break;
            }
            output.extend_from_slice(&buffer[..count]);
            if !ready
                && (output.windows(6).any(|bytes| bytes == b"INSERT")
                    || output.windows(6).any(|bytes| bytes == b"NORMAL"))
            {
                ready = true;
                ready_sender.send(()).expect("signal rendered prompt");
            }
        }
        sender.send(output).expect("send PTY output");
    });

    let mut writer = pair.master.take_writer().expect("open PTY writer");
    // Ratatui's inline viewport asks the terminal for its current cursor row.
    writer
        .write_all(b"\x1b[1;1R")
        .expect("answer cursor-position query");
    writer.flush().expect("flush cursor-position answer");
    ready_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("prompt should render before input");
    writer.write_all(keys).expect("send prompt keys");
    writer.flush().expect("flush prompt keys");
    let status = child.wait().expect("wait for prompt").exit_code();
    drop(writer);
    drop(pair.master);

    PromptResult {
        status,
        stdout: fs::read(stdout).expect("read clean stdout"),
        terminal: receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("collect PTY output"),
    }
}

fn shell_word(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[test]
fn cli_001_accept_single_line_input() {
    let result = prompt("exec {ink} input", b"release title\r");

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"release title\n");
    assert!(!result.stdout.contains(&0x1b));
    assert!(result.terminal.windows(5).any(|bytes| bytes == b"\x1b[0 q"));
    assert!(
        result
            .terminal
            .windows(6)
            .any(|bytes| bytes == b"\x1b[?25h")
    );
}

#[test]
fn cli_002_reject_line_breaks_in_input() {
    let result = prompt(
        "exec {ink} input",
        b"\x1b[200~first\nsecond\r\nthird\x1b[201~\r",
    );

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"firstsecondthird\n");
}

#[test]
fn cli_003_accept_multiline_text() {
    let result = prompt("exec {ink} textarea", b"first\rsecond\x04");

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"first\nsecond\n");
}

#[test]
fn cli_004_default_to_insert_mode() {
    let result = prompt("exec {ink} input --value text", b"\x03");

    assert_eq!(
        result.status,
        130,
        "terminal: {}",
        String::from_utf8_lossy(&result.terminal).escape_debug()
    );
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.terminal).contains("INSERT"));
    assert!(result.terminal.windows(5).any(|bytes| bytes == b"\x1b[6 q"));
    assert!(
        result
            .terminal
            .windows(8)
            .any(|bytes| bytes == b"\x1b[?2004l")
    );
    assert!(
        result
            .terminal
            .windows(6)
            .any(|bytes| bytes == b"\x1b[?25h")
    );
}

#[test]
fn cli_005_start_in_normal_mode() {
    let result = prompt("exec {ink} input --normal --value unchanged", b"\x04");

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"unchanged\n");
    assert!(String::from_utf8_lossy(&result.terminal).contains("NORMAL"));
    assert!(result.terminal.windows(5).any(|bytes| bytes == b"\x1b[2 q"));
}

#[test]
fn cli_006_seed_from_command_line() {
    let result = prompt("exec {ink} input --value seed", b"!\r");

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"seed!\n");
}

#[test]
fn cli_007_seed_from_piped_input() {
    let result = prompt("printf 'first\\nsecond' | {ink} input", b"\x04");

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"firstsecond\n");
}

#[test]
fn cli_008_command_line_seed_wins() {
    let result = prompt("printf ignored | {ink} input --value explicit", b"\x04");

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"explicit\n");
}

#[test]
fn cli_009_configure_the_input_prompt() {
    let result = prompt("exec {ink} input", b"value\r");
    assert_eq!(result.stdout, b"value\n");
    assert!(!result.terminal.windows(2).any(|bytes| bytes == b"> "));

    let result = prompt("exec {ink} input --prompt 'name: '", b"Dorian\r");
    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, b"Dorian\n");
    assert!(result.terminal.windows(6).any(|bytes| bytes == b"name: "));

    let result = prompt("exec {ink} input --prompt ''", b"value\r");
    assert_eq!(result.stdout, b"value\n");
    assert!(!result.terminal.windows(2).any(|bytes| bytes == b"> "));
}

#[test]
fn cli_010_keep_textarea_compact_by_default() {
    let result = prompt("exec {ink} textarea", b"\x03");

    assert_eq!(result.status, 130);
    assert!(result.terminal.windows(4).any(|bytes| bytes == b"\x1b[6;"));
    assert!(!result.terminal.windows(4).any(|bytes| bytes == b"\x1b[7;"));
}

#[test]
fn cli_011_expand_textarea_to_full_screen() {
    let result = prompt("exec {ink} textarea --fullscreen", b"\x03");

    assert_eq!(result.status, 130);
    assert!(result.terminal.windows(4).any(|bytes| bytes == b"\x1b[8;"));
}

#[test]
fn cli_012_remove_prompt_ui_after_completion() {
    for (command, keys) in [
        ("exec {ink} input --value accepted", b"\r".as_slice()),
        ("exec {ink} textarea", b"\x03".as_slice()),
    ] {
        let result = prompt(command, keys);
        assert!(
            result
                .terminal
                .ends_with(b"\x1b8\x1b[J\x1b[0 q\x1b[?2004l\x1b[?25h")
        );
    }
}
