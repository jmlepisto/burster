//! Fixed window -type limiter

#[cfg(feature = "std")]
use crate::macros::std_time_provider;
use crate::{CantConsume, LimiterResult};

/// Build a fixed window limiter
///
/// # Arguments
/// * `capacity` - how many consumes are allowed during a single window
/// * `window_width_ms` - window width as milliseconds
#[cfg(feature = "std")]
pub fn fixed_window(capacity: u64, window_width_ms: u64) -> FixedWindow<impl Fn() -> u64> {
    FixedWindow::new_with_time_provider(capacity, window_width_ms, std_time_provider!())
}

/// Fixed window -type rate limiter
///
/// A Fixed window limiter splits the timeline into time windows
/// of defined size and allocates a certain amount of tokens for
/// each window. Consumes are successfull as long as the current
/// time window still holds enought tokens.
pub struct FixedWindow<T>
where
    T: Fn() -> u64,
{
    config: FixedWindowConfig<T>,
    tokens: u64,
    window_index: u64,
    start_time: u64,
}

impl<T> FixedWindow<T>
where
    T: Fn() -> u64,
{
    /// Initialize a new fixed window limiter utilizing the given timer
    ///
    /// # Arguments
    /// * `capacity` - how many consumes are allowed during a single window
    /// * `window_width_ms` - window width as milliseconds
    /// * `time_provider_t` - closure that returns a monotonically nondecreasing
    ///   timestamp as u64 milliseconds
    ///
    /// If you are developing for a `std` target, you probably wish to use [`fixed_window`]
    pub fn new_with_time_provider(capacity: u64, window_width_ms: u64, time_provider: T) -> Self {
        let time_now = time_provider();
        let config = FixedWindowConfig::new(capacity, window_width_ms, time_provider);
        Self {
            config,
            tokens: capacity,
            window_index: 0,
            start_time: time_now,
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
        // Get current window index
        let now = (self.config.time_provider)();
        let delta_t = now.saturating_sub(self.start_time);
        let index = delta_t / self.config.width_ms;

        if index != self.window_index {
            // New window. Replenish tokens.
            self.tokens = self.config.capacity;
            self.window_index = index;
        }

        self.tokens = self.tokens.checked_sub(tokens).ok_or(CantConsume)?;
        Ok(())
    }
}

/// Configuration for a fixed window limiter
#[derive(Clone, Copy)]
struct FixedWindowConfig<T>
where
    T: Fn() -> u64,
{
    capacity: u64,
    width_ms: u64,
    time_provider: T,
}

impl<T: Fn() -> u64> FixedWindowConfig<T> {
    fn new(capacity: u64, width_ms: u64, time_provider: T) -> Self {
        Self {
            capacity,
            width_ms,
            time_provider,
        }
    }
}
