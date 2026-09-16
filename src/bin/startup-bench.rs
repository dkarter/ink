#![forbid(unsafe_code)]

use std::{
    env,
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};

const WARMUPS: usize = 20;
const SAMPLES: usize = 100;
const BUDGET_MS: f64 = 20.0;

fn main() {
    let binary = env::args()
        .nth(1)
        .unwrap_or_else(|| "target/release/ink".into());
    let binary = Path::new(&binary);
    if !binary.is_file() {
        eprintln!("release binary not found at {}", binary.display());
        std::process::exit(2);
    }

    for _ in 0..WARMUPS {
        probe(binary);
    }

    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        probe(binary);
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    let median_ms =
        (samples[SAMPLES / 2 - 1].as_secs_f64() + samples[SAMPLES / 2].as_secs_f64()) * 500.0;
    println!("ink --version median: {median_ms:.3} ms ({WARMUPS} warmups, {SAMPLES} samples)");

    if env::var_os("INK_ENFORCE_STARTUP_BUDGET").is_some() && median_ms > BUDGET_MS {
        eprintln!("startup median exceeds {BUDGET_MS:.1} ms budget");
        std::process::exit(1);
    }
}

fn probe(binary: &Path) {
    let status = Command::new(binary)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", binary.display()));
    assert!(status.success(), "startup probe failed with {status}");
}
