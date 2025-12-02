//! Enforcement rules for state transitions using Validation.

use crate::core::State;
use crate::enforcement::context::TransitionContext;
use crate::enforcement::violations::{ViolationError, ViolationStrategy};
use std::time::Duration;
use stillwater::validation::Validation;
use stillwater::NonEmptyVec;

/// Type alias for validation check functions
pub type ValidationCheck<S> =
    Box<dyn Fn(&TransitionContext<S>) -> Validation<(), NonEmptyVec<ViolationError>> + Send + Sync>;

/// Enforcement rules for state transitions.
/// Uses Validation to accumulate ALL violations.
pub struct EnforcementRules<S: State> {
    pub(crate) max_attempts: Option<usize>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) required_checks: Vec<ValidationCheck<S>>,
    pub(crate) on_violation: ViolationStrategy,
}

impl<S: State> EnforcementRules<S> {
    /// Enforce all rules, accumulating ALL violations.
    /// Returns Validation::Success(()) if all checks pass.
    /// Returns Validation::Failure with ALL violations if any fail.
    pub fn enforce(
        &self,
        context: &TransitionContext<S>,
    ) -> Validation<(), NonEmptyVec<ViolationError>> {
        let mut checks: Vec<Validation<(), NonEmptyVec<ViolationError>>> = Vec::new();

        // Check max attempts
        if let Some(max) = self.max_attempts {
            let check = if context.attempt > max {
                Validation::fail(ViolationError::MaxAttemptsExceeded {
                    max,
                    current: context.attempt,
                })
            } else {
                Validation::success(())
            };
            checks.push(check);
        }

        // Check timeout
        if let Some(timeout) = self.timeout {
            let elapsed = context.elapsed();
            let check = if elapsed > timeout {
                Validation::fail(ViolationError::TimeoutExceeded { timeout, elapsed })
            } else {
                Validation::success(())
            };
            checks.push(check);
        }

        // Run custom checks
        for check_fn in &self.required_checks {
            checks.push(check_fn(context));
        }

        // Accumulate ALL failures using all_vec
        Validation::all_vec(checks).map(|_| ())
    }

    pub fn violation_strategy(&self) -> ViolationStrategy {
        self.on_violation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enforcement::builder::EnforcementBuilder;
    use chrono::Utc;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum TestState {
        Initial,
        Processing,
        Complete,
    }

    impl State for TestState {
        fn name(&self) -> &str {
            match self {
                Self::Initial => "Initial",
                Self::Processing => "Processing",
                Self::Complete => "Complete",
            }
        }

        fn is_final(&self) -> bool {
            matches!(self, Self::Complete)
        }
    }

    #[test]
    fn enforcement_accumulates_all_violations() {
        let rules = EnforcementBuilder::new()
            .max_attempts(3)
            .timeout(Duration::from_secs(5))
            .require_pred(|_ctx| false, "Custom check always fails".to_string())
            .build();

        let context = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 5,
            started_at: Utc::now() - chrono::Duration::seconds(10),
        };

        let result = rules.enforce(&context);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 3);

                let has_max_attempts = errors
                    .iter()
                    .any(|e| matches!(e, ViolationError::MaxAttemptsExceeded { .. }));
                let has_timeout = errors
                    .iter()
                    .any(|e| matches!(e, ViolationError::TimeoutExceeded { .. }));
                let has_custom = errors
                    .iter()
                    .any(|e| matches!(e, ViolationError::CustomCheckFailed { .. }));

                assert!(has_max_attempts);
                assert!(has_timeout);
                assert!(has_custom);
            }
            Validation::Success(_) => panic!("Expected failures, got success"),
        }
    }

    #[test]
    fn enforcement_succeeds_when_all_checks_pass() {
        let rules = EnforcementBuilder::new()
            .max_attempts(10)
            .timeout(Duration::from_secs(60))
            .require_pred(|_ctx| true, "This check always passes".to_string())
            .build();

        let context = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 1,
            started_at: Utc::now(),
        };

        let result = rules.enforce(&context);
        assert!(result.is_success());
    }

    #[test]
    fn custom_validation_check_works() {
        let rules = EnforcementBuilder::new()
            .require(|ctx: &TransitionContext<TestState>| {
                if ctx.attempt > 0 {
                    Validation::success(())
                } else {
                    Validation::fail(ViolationError::CustomCheckFailed {
                        message: "Attempt must be > 0".to_string(),
                    })
                }
            })
            .build();

        let context = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 0,
            started_at: Utc::now(),
        };

        let result = rules.enforce(&context);
        assert!(result.is_failure());
    }

    #[test]
    fn max_attempts_enforcement() {
        let rules = EnforcementBuilder::new().max_attempts(3).build();

        let context_pass = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 2,
            started_at: Utc::now(),
        };

        assert!(rules.enforce(&context_pass).is_success());

        let context_fail = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 4,
            started_at: Utc::now(),
        };

        let result = rules.enforce(&context_fail);
        assert!(result.is_failure());
        if let Validation::Failure(errors) = result {
            assert!(errors
                .iter()
                .any(|e| matches!(e, ViolationError::MaxAttemptsExceeded { .. })));
        }
    }

    #[test]
    fn timeout_enforcement() {
        let rules = EnforcementBuilder::new()
            .timeout(Duration::from_secs(1))
            .build();

        let context_pass = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 1,
            started_at: Utc::now(),
        };

        assert!(rules.enforce(&context_pass).is_success());

        let context_fail = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 1,
            started_at: Utc::now() - chrono::Duration::seconds(5),
        };

        let result = rules.enforce(&context_fail);
        assert!(result.is_failure());
        if let Validation::Failure(errors) = result {
            assert!(errors
                .iter()
                .any(|e| matches!(e, ViolationError::TimeoutExceeded { .. })));
        }
    }

    #[test]
    fn violation_strategy_is_stored() {
        let rules: EnforcementRules<TestState> = EnforcementBuilder::new()
            .on_violation(ViolationStrategy::Retry)
            .build();

        assert_eq!(rules.violation_strategy(), ViolationStrategy::Retry);
    }
}
