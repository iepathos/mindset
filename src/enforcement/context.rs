//! Context provided to enforcement checks.

use crate::core::State;
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Context provided to enforcement checks
#[derive(Clone, Debug)]
pub struct TransitionContext<S: State> {
    pub from: S,
    pub to: S,
    pub attempt: usize,
    pub started_at: DateTime<Utc>,
}

impl<S: State> TransitionContext<S> {
    /// Calculate elapsed time since transition started (pure)
    pub fn elapsed(&self) -> Duration {
        let now = Utc::now();
        now.signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or(Duration::ZERO)
    }
}
