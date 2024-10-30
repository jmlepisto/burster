//! Sliding window -type limiter

#[cfg(feature = "std")]
use crate::macros::std_time_provider;
use crate::{CantConsume, LimiterResult};

/// Build a sliding window limiter
///
/// Window width is defined by the generic argument `W: usize`
///
/// # Arguments
/// * `capacity` - how many consumes are allowed during a single window
#[cfg(feature = "std")]
pub fn sliding_window<const W: usize>(capacity: u64) -> SlidingWindow<impl Fn() -> u64, W> {
    SlidingWindow::<_, W>::new_with_time_provider(capacity, std_time_provider!())
}

/// Sliding window -type rate limiter
///
/// A sliding windows limiter keeps track of tokens used
/// during the last `window_width` milliseconds before the
/// most recent consume and limits usage if that number grows
/// larger than the defined limit.
///
/// # Generic arguments
/// * `W` - Window width in milliseconds
pub struct SlidingWindow<T, const W: usize>
where
    T: Fn() -> u64,
{
    config: SlidingWindowConfig<T>,
    /// Each slot represents a point in past time relative to current time.
    /// When time moves forward, we effectively shift the slots to right.
    window_buffer: [u64; W],
    last_check_time_ms: u64,
}

impl<T, const W: usize> SlidingWindow<T, W>
where
    T: Fn() -> u64,
{
    /// Initialize a new sliding window limiter utilizing the given timer
    ///
    /// # Arguments
    /// * `capacity` - how many consumes are allowed during a single window
    /// * `time_provider_t` - closure that returns a monotonically nondecreasing
    ///   timestamp as u64 milliseconds
    ///
    /// # Notes
    /// * If you are developing for a `std` target, you probably wish to use [`sliding_window`]
    /// * Window width is defined by the generic argument `W: usize`
    pub fn new_with_time_provider(capacity: u64, time_provider: T) -> Self {
        let time_now = time_provider();
        let config = SlidingWindowConfig::new(capacity, time_provider);
        Self {
            config,
            window_buffer: [0; W],
            last_check_time_ms: time_now,
        }
    }

    /// Try to consume a single token
    ///
    /// # Returns
    /// * `Ok(())` - token consumed
    /// * `Err(())` - not enough tokens left for this time window
    pub fn try_consume_one(&mut self) -> LimiterResult {
        self.try_consume(1)
    }

    /// Try to consume tokens
    ///
    /// # Arguments
    /// * `tokens` - how many tokens to consume
    ///
    /// # Returns
    /// * `Ok(())` - token consumed
    /// * `Err(())` - not enough tokens left for this time window
    pub fn try_consume(&mut self, tokens: u64) -> LimiterResult {
        let now = (self.config.time_provider)();
        let delta_t = now.saturating_sub(self.last_check_time_ms);
        self.last_check_time_ms = now;

        // delta_t is more than the window size, reset the whole limiter
        if delta_t >= W as u64 {
            self.window_buffer.fill(0);
            self.window_buffer[0] = tokens;
            return Ok(());
        }

        // Time has moved on, shift existing items right for delta_t slots
        let move_range = 0..(W - delta_t as usize);
        self.window_buffer.copy_within(move_range, delta_t as usize);

        // Zero all slots that were not updated
        self.window_buffer[..delta_t as usize].fill(0);

        // Too many tokens used during the window?
        let tokens_left = self.config.capacity - self.window_buffer.iter().sum::<u64>();
        if tokens_left >= tokens {
            // Add tokens to current timeslot
            self.window_buffer[0] += tokens;
            Ok(())
        } else {
            Err(CantConsume)
        }
    }
}

/// Configuration for a fixed window limiter
#[derive(Clone, Copy)]
struct SlidingWindowConfig<T>
where
    T: Fn() -> u64,
{
    capacity: u64,
    time_provider: T,
}

impl<T: Fn() -> u64> SlidingWindowConfig<T> {
    fn new(capacity: u64, time_provider: T) -> Self {
        Self {
            capacity,
            time_provider,
        }
    }
}
