//! Deterministic startup benchmark protocol.

use std::time::Duration;

pub const WARMUPS: usize = 20;
pub const SAMPLES: usize = 100;
pub const BUDGET: Duration = Duration::from_millis(20);

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
