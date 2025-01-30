use std::cmp::min;

/// A utility for implementing exponential backoff retry logic
#[derive(Debug, Clone)]
pub struct Backoff {
    /// Maximum number of retry attempts allowed
    retries: u32,
    /// Minimum backoff duration in seconds
    min_secs: u32,
    /// Maximum backoff duration in seconds
    max_secs: u32,
    /// Multiplication factor for exponential increase
    factor: u32,
    /// Current retry attempt counter
    counter: u32,
    /// Current backoff duration in seconds
    value_secs: u32,
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new(10, 1, 20, 2)
    }
}

impl Backoff {
    pub fn new(retries: u32, min_secs: u32, max_secs: u32, factor: u32) -> Self {
        Self {
            retries,
            min_secs,
            max_secs,
            factor,
            counter: 0,
            value_secs: min_secs,
        }
    }

    /// Resets the backoff counter and duration
    pub fn reset(&mut self) {
        self.counter = 0;
        self.value_secs = self.min_secs;
    }

    /// Get the current value for the retry counter
    pub fn get_iteration_count(&self) -> u32 {
        self.counter
    }
}

impl Iterator for Backoff {
    type Item = u32;

    /// Get the next backoff duration in seconds for next retry attempt
    fn next(&mut self) -> Option<Self::Item> {
        if self.retries > 0 && self.counter >= self.retries {
            return None;
        }

        let value = self.value_secs;
        self.counter += 1;
        self.value_secs = match self.counter {
            1 => self.min_secs,
            _ => min(self.value_secs * self.factor, self.max_secs),
        };
        Some(value)
    }
}
