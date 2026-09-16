#![forbid(unsafe_code)]

use std::{
    env,
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};

use ink::startup_bench::{
    BUDGET, ENFORCE_BUDGET_ENV, SAMPLE_FIXTURE_ENV, SAMPLES, WARMUPS, budget_exceeded, measure,
    measure_fixture,
};

fn main() {
    let median = if let Some(fixture) = env::var_os(SAMPLE_FIXTURE_ENV) {
        measure_fixture(Path::new(&fixture)).unwrap_or_else(|error| {
            eprintln!("invalid startup sample fixture: {error}");
            std::process::exit(2);
        })
    } else {
        measure_binary()
    };
    let median_ms = median.as_secs_f64() * 1_000.0;
    println!("ink --version median: {median_ms:.3} ms ({WARMUPS} warmups, {SAMPLES} samples)");

    if budget_exceeded(env::var_os(ENFORCE_BUDGET_ENV).is_some(), median) {
        eprintln!(
            "startup median exceeds {:.1} ms budget",
            BUDGET.as_secs_f64() * 1_000.0
        );
        std::process::exit(1);
    }
}

fn measure_binary() -> std::time::Duration {
    let binary = env::args_os()
        .nth(1)
        .unwrap_or_else(|| "target/release/ink".into());
    let binary = Path::new(&binary);
    if !binary.is_file() {
        eprintln!("release binary not found at {}", binary.display());
        std::process::exit(2);
    }

    measure(|| {
        let started = Instant::now();
        probe(binary);
        started.elapsed()
    })
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
