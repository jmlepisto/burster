# Burster ⏩

![Crates.io Version](https://img.shields.io/crates/v/burster?style=flat-square&link=https%3A%2F%2Fcrates.io%2Fcrates%2Fburster)
![docs.rs](https://img.shields.io/docsrs/burster?style=flat-square&link=https%3A%2F%2Fdocs.rs%2Fburster%2Flatest%2Fburster%2F)


Burster is a high quality and lightweigh crate providing stack allocated rate limiters with minimal dependencies.
Guaranteed to work on `no_std` targets, but also comfortable on standard targets.

## Supported rate limiter types

- Token bucket
- Fixed window
- Sliding window
- ..something else? Make a request or open a PR :)

## Usage

On `std` targets usage is simple. Install the crate with default features enabled and
you'll get access to straightforward utility functions for instantiating limiters.

```rust
// Instantiate a token bucket that allowes an average consume
// rate of 100 tokens per second and with bucket_size = 10
let mut bucket = burster::token_bucket(100, 10);

// Use the bucket:
if bucket.try_consume_one().is_ok() {
    // All good, enough tokens left
} else {
    // Not enough tokens for this consume
}
```

On `no_std` targets you'll have to install the crate with default features disabled and
provide bindings to your platforms clock functionality in the form of a closure that returns
the current timestamp as `u64` milliseconds.

```rust
// Instantiate a token bucket that allowes an average consume
// rate of 100 tokens per second and with bucket_size = 10
let mut bucket = burster::TokenBucket::new_with_time_provider(100, 10, || {
    // Return current timestamp
    my_clock_fn()
});
```

The *time provider* closure should return a monotonous nondecreasing timestamp, which does
not have to be bound to a specific epoch. It can, for example, simply be a timestamp from
the last system boot.