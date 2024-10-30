//! Token bucket -type limiter

#[cfg(feature = "std")]
use crate::macros::std_time_provider;
use crate::{CantConsume, LimiterResult};

/// Build a token bucket limiter
///
/// # Arguments
/// * `rate_per_sec` - how many consumes should be allowed per second on average
/// * `capacity` - bucket capacity to dictate the burstiness of this limiter
#[cfg(feature = "std")]
pub fn token_bucket(rate_per_s: u64, capacity: u64) -> TokenBucket<impl Fn() -> u64> {
    TokenBucket::new_with_time_provider(rate_per_s, capacity, std_time_provider!())
}

/// Token bucket -type rate limiter
///
/// A token bucket limiter can be illustrated as a being filled
/// with tokens at a constant rate, while consumes will remove
/// tokens from the bucket.
///
/// This leads to a soft limiter where occasional burstiness is
/// allowed since as long as the bucket holds tokens those can
/// be consumed at an unlimited rate. Ultimately the bucket size
/// is what defined the burstiness.
pub struct TokenBucket<T>
where
    T: Fn() -> u64,
{
    config: TokenBucketConfig<T>,
    tokens: u64,
    last_update_t_ms: u64,
}

impl<T> TokenBucket<T>
where
    T: Fn() -> u64,
{
    /// Initialize a new token bucket utilizing the given timer
    ///
    /// # Arguments
    /// * `rate_per_sec` - how many consumes should be allowed per second on average
    /// * `capacity` - bucket capacity to dictate the burstiness of this limiter
    /// * `time_provider_t` - closure that returns a monotonically nondecreasing
    ///   timestamp as u64 milliseconds
    ///
    /// If you are developing for a `std` target, you probably wish to use [`token_bucket`]
    pub fn new_with_time_provider(rate_per_s: u64, capacity: u64, time_provider: T) -> Self {
        let time_now = time_provider();
        let config = TokenBucketConfig::new(capacity, rate_per_s, time_provider);
        Self {
            config,
            tokens: capacity,
            last_update_t_ms: time_now,
        }
    }

    /// Try to consume a single token from the bucket
    ///
    /// # Returns
    /// * `Ok(())` - token consumed
    /// * `Err(())` - not enough tokens in the bucket
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
    /// * `Err(())` - not enough tokens in the bucket
    fn try_consume(&mut self, tokens: u64) -> LimiterResult {
        // First, get elapsed time since last call
        let now = (self.config.time_provider)();
        let delta_t = now.saturating_sub(self.last_update_t_ms);

        if delta_t != 0 {
            let tokens_to_add = (0.001 * delta_t as f64 * self.config.rate_per_s) as u64;

            // If the tokens to add rounds down to zero, lets not update
            // the timestamp so we don't lose any accumulated tokens due
            // to rounding inaccuracies.
            if tokens_to_add != 0 {
                self.last_update_t_ms = now;
                self.tokens = (self.tokens.saturating_add(tokens_to_add)).min(self.config.capacity);
            }
        }

        // Take away tokens, if possible
        self.tokens = self.tokens.checked_sub(tokens).ok_or(CantConsume)?;
        Ok(())
    }
}

/// Configuration for a token bucket
#[derive(Clone, Copy)]
struct TokenBucketConfig<T>
where
    T: Fn() -> u64,
{
    capacity: u64,
    rate_per_s: f64,
    time_provider: T,
}

impl<T: Fn() -> u64> TokenBucketConfig<T> {
    fn new(capacity: u64, rate_per_s: u64, time_provider: T) -> Self {
        Self {
            capacity,
            rate_per_s: rate_per_s as f64,
            time_provider,
        }
    }
}
