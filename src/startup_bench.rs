//! Deterministic startup benchmark protocol.

use std::{fs, io, path::Path, time::Duration};

pub const WARMUPS: usize = 20;
pub const SAMPLES: usize = 100;
pub const BUDGET: Duration = Duration::from_millis(20);
pub const ENFORCE_BUDGET_ENV: &str = "INK_ENFORCE_STARTUP_BUDGET";
pub const SAMPLE_FIXTURE_ENV: &str = "INK_STARTUP_BENCH_SAMPLE_FIXTURE";

pub fn measure(mut probe: impl FnMut() -> Duration) -> Duration {
    for _ in 0..WARMUPS {
        probe();
    }

    let mut samples = [Duration::ZERO; SAMPLES];
    for sample in &mut samples {
        *sample = probe();
    }
    samples.sort_unstable();

    (samples[SAMPLES / 2 - 1] + samples[SAMPLES / 2]) / 2
}

pub fn budget_exceeded(enforce: bool, median: Duration) -> bool {
    enforce && median > BUDGET
}

pub fn measure_fixture(path: &Path) -> io::Result<Duration> {
    let fixture = fs::read_to_string(path)?;
    let samples = fixture
        .lines()
        .map(|line| {
            line.parse::<u64>()
                .map(Duration::from_nanos)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        })
        .collect::<io::Result<Vec<_>>>()?;
    if samples.len() != WARMUPS + SAMPLES {
        return Err(invalid_fixture_count());
    }
    let mut samples = samples.into_iter();
    Ok(measure(|| {
        samples.next().expect("fixture length was validated")
    }))
}

fn invalid_fixture_count() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "sample fixture must contain exactly {} values",
            WARMUPS + SAMPLES
        ),
    )
}
