//! Builder API for creating enforcement rules.

use crate::core::State;
use crate::enforcement::context::TransitionContext;
use crate::enforcement::rules::{EnforcementRules, ValidationCheck};
use crate::enforcement::violations::{ViolationError, ViolationStrategy};
use std::time::Duration;
use stillwater::validation::Validation;
use stillwater::NonEmptyVec;

/// Builder for creating enforcement rules
pub struct EnforcementBuilder<S: State> {
    max_attempts: Option<usize>,
    timeout: Option<Duration>,
    required_checks: Vec<ValidationCheck<S>>,
    on_violation: ViolationStrategy,
}

impl<S: State> EnforcementBuilder<S> {
    pub fn new() -> Self {
        Self {
            max_attempts: None,
            timeout: None,
            required_checks: Vec::new(),
            on_violation: ViolationStrategy::Abort,
        }
    }

    /// Set maximum retry attempts
    pub fn max_attempts(mut self, n: usize) -> Self {
        self.max_attempts = Some(n);
        self
    }

    /// Set timeout duration
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Add a custom validation check
    pub fn require<F>(mut self, check: F) -> Self
    where
        F: Fn(&TransitionContext<S>) -> Validation<(), NonEmptyVec<ViolationError>>
            + Send
            + Sync
            + 'static,
    {
        self.required_checks.push(Box::new(check));
        self
    }

    /// Add a simple predicate check with error message
    pub fn require_pred<F>(mut self, predicate: F, error_msg: String) -> Self
    where
        F: Fn(&TransitionContext<S>) -> bool + Send + Sync + 'static,
    {
        let check = move |ctx: &TransitionContext<S>| {
            if predicate(ctx) {
                Validation::success(())
            } else {
                Validation::fail(ViolationError::CustomCheckFailed {
                    message: error_msg.clone(),
                })
            }
        };
        self.required_checks.push(Box::new(check));
        self
    }

    /// Set violation handling strategy
    pub fn on_violation(mut self, strategy: ViolationStrategy) -> Self {
        self.on_violation = strategy;
        self
    }

    /// Build the enforcement rules
    pub fn build(self) -> EnforcementRules<S> {
        EnforcementRules {
            max_attempts: self.max_attempts,
            timeout: self.timeout,
            required_checks: self.required_checks,
            on_violation: self.on_violation,
        }
    }
}

impl<S: State> Default for EnforcementBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
