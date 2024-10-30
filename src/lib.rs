//! # Burster
//!
//! Burster is a lightweigh crate providing stack allocated rate limiters
//! with minimal dependencies.
//!
//! ## Available limiters
//!
//! * [`TokenBucket`] - basic token bucket type limiter
//! * [`FixedWindow`] - fixed window type limiter
//! * [`SlidingWindow`] - sliding window type limiter
//!
//! ## Platform support
//!
//! On `std` targets you are all good to go and can use the following utility
//! functions for instantiating the limiters:
//!
//! * [`token_bucket`]
//! * [`fixed_window`]
//! * [`sliding_window`]
//!
//! On `no_std` targets you'll have to provide bindings to your platforms timing
//! functionalities and use the constructor methods:
//!
//! * [`TokenBucket::new_with_time_provider`]
//! * [`FixedWindow::new_with_time_provider`]
//! * [`SlidingWindow::new_with_time_provider`]
//!
//! You must provide timer access in the form of a closuse that returns current system
//! timestamp as `u64` milliseconds.

// Support no_std
#![cfg_attr(not(feature = "std"), no_std)]

mod fixed_window_impl;
mod sliding_window_impl;
mod token_bucket_impl;

#[cfg(feature = "std")]
#[cfg(test)]
mod test;

use core::fmt;

#[cfg(feature = "std")]
pub use token_bucket_impl::token_bucket;
pub use token_bucket_impl::TokenBucket;

#[cfg(feature = "std")]
pub use fixed_window_impl::fixed_window;
pub use fixed_window_impl::FixedWindow;

#[cfg(feature = "std")]
pub use sliding_window_impl::sliding_window;
pub use sliding_window_impl::SlidingWindow;

/// Error type indicating that the requested amount of
/// tokens cannot be consumed from the limiter.
///
/// I.e. the limiter *limits*
#[derive(Debug)]
pub struct CantConsume;

impl fmt::Display for CantConsume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Can't consume from limiter")
    }
}

// core::error::Error trait stabilised at release 1.81
#[rustversion::since(1.81)]
impl core::error::Error for CantConsume {}

/// Limiter consume action result type
///
/// There are no actual errors that can be returned,
/// and the error type here is only used for signalling
/// that the requested amount of tokens cannot be consumed.
pub type LimiterResult = Result<(), CantConsume>;

#[cfg(feature = "std")]
mod macros {
    macro_rules! std_time_provider {
        () => {
            || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis() as u64
            }
        };
    }

    pub(crate) use std_time_provider;
}
