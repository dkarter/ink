#![forbid(unsafe_code)]

mod specs {
    fn wait_for_child(
        child: &mut (dyn portable_pty::Child + Send + Sync),
        description: &str,
    ) -> u32 {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if let Some(status) = child.try_wait().expect("poll PTY child") {
                return status.exit_code();
            }
            if std::time::Instant::now() >= deadline {
                child.kill().expect("kill stuck PTY child");
                child.wait().expect("reap stuck PTY child");
                panic!("{description} did not exit within three seconds");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn sequence_position(haystack: &[u8], needle: &[u8]) -> usize {
        haystack
            .windows(needle.len())
            .position(|bytes| bytes == needle)
            .unwrap_or_else(|| panic!("missing sequence: {:?}", String::from_utf8_lossy(needle)))
    }

    mod command_line;
    mod completions_and_performance;
    mod configuration;
    mod placeholders;
    mod presentation;
    mod releases;
    mod terminal_io;
    mod themes;
    mod vim_editing;
}
