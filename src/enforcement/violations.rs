//! Violation errors and handling strategies.

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur when enforcing transition policies
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ViolationError {
    #[error("Maximum attempts ({max}) exceeded (current: {current})")]
    MaxAttemptsExceeded { max: usize, current: usize },

    #[error("Timeout ({timeout:?}) exceeded (elapsed: {elapsed:?})")]
    TimeoutExceeded {
        timeout: Duration,
        elapsed: Duration,
    },

    #[error("Custom check failed: {message}")]
    CustomCheckFailed { message: String },
}

/// Strategy for handling enforcement violations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViolationStrategy {
    /// Abort transition permanently
    Abort,

    /// Allow retry despite violation
    Retry,

    /// Continue but log warning
    IgnoreAndLog,
}
