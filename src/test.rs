use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

use crate::FixedWindow;
use crate::SlidingWindow;
use crate::TokenBucket;

struct MockClock(AtomicU64);

impl MockClock {
    pub fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn step(&self, step: u64) -> u64 {
        self.0.fetch_add(step, Ordering::Relaxed)
    }
}

#[test]
fn test_token_bucket_average_rate() {
    let rate = 100;
    let epsilon = 5.0;
    let clock = MockClock::new();
    let n = 200_000;
    let bucket_sizes = [1, 100, 1000];

    for s in bucket_sizes {
        // Fake timekeeping increments our "clock" by 1ms each call
        let mut b = TokenBucket::new_with_time_provider(rate, s, || clock.step(1));

        let mut pass = 0;
        for _ in 0..n {
            if b.try_consume_one().is_ok() {
                pass += 1;
            }
        }

        let seconds = n as f64 * 0.001;
        let tokens_per_second = pass as f64 / seconds;
        println!("{tokens_per_second}");
        assert!((tokens_per_second - rate as f64).abs() <= epsilon);
    }
}

#[test]
fn test_fixed_window_average_rate() {
    let n = 200_000;
    let epsilon = 0.01;
    let rate = 100;
    let clock = MockClock::new();

    // Fake timekeeping increments our "clock" by 1ms each call
    // 100 tokens per 1000 milliseconds should add up to a rate of 100/s.
    let mut w = FixedWindow::new_with_time_provider(100, 1000, || clock.step(1));

    let mut n_pass = 0;
    for _ in 0..n {
        if w.try_consume_one().is_ok() {
            n_pass += 1;
        }
    }

    let seconds = n as f64 * 0.001;
    let tokens_per_second = n_pass as f64 / seconds;
    assert!((tokens_per_second - rate as f64).abs() <= epsilon);
}

#[test]
fn test_sliding_window_average_rate() {
    let n = 200_000;
    let epsilon = 0.01;
    let rate = 100;
    let clock = MockClock::new();

    // Fake timekeeping increments our "clock" by 1ms each call
    // 100 tokens per 1000 milliseconds should add up to a rate of 100/s.
    let mut w = SlidingWindow::<_, 1000>::new_with_time_provider(100, || clock.step(1));

    let mut n_pass = 0;
    for _ in 0..n {
        if w.try_consume_one().is_ok() {
            n_pass += 1;
        }
    }

    let seconds = n as f64 * 0.001;
    let tokens_per_second = n_pass as f64 / seconds;
    assert!((tokens_per_second - rate as f64).abs() <= epsilon);
}
