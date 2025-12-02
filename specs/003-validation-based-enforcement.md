---
number: 003
title: Validation-Based Enforcement System
category: foundation
priority: high
status: draft
dependencies: [001, 002]
created: 2025-12-01
---

# Specification 003: Validation-Based Enforcement System

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Specification 001, Specification 002

## Context

State machines often need to enforce policies and constraints on transitions:
- Maximum retry attempts before giving up
- Timeout limits for long-running transitions
- Custom validation checks that must pass
- Multiple independent checks that should ALL be reported

Current approaches fail-fast, showing only the first violation. This frustrates users who must fix errors one at a time. Stillwater's `Validation` type solves this by accumulating ALL errors.

Following Stillwater's philosophy: "Don't stop at first error - collect them all!"

## Objective

Implement an enforcement system using Stillwater's `Validation` type to accumulate ALL policy violations, providing comprehensive feedback for transition failures. Use Stillwater's predicate combinators for composable validation logic.

## Requirements

### Functional Requirements

- **Enforcement Rules**: Define policies for transition constraints
- **Error Accumulation**: Use `Validation` to collect ALL violations
- **Predicate Combinators**: Use Stillwater predicates for checks
- **Max Attempts**: Enforce maximum retry limits
- **Timeout Checks**: Enforce time-based constraints
- **Custom Checks**: Support user-defined validation predicates
- **Violation Strategies**: Handle violations (Abort, Retry, Log)
- **Integration**: Attach enforcement to transitions

### Non-Functional Requirements

- **Fail Completely**: Return ALL violations, not just first
- **Pure Checks**: Enforcement checks are pure predicates
- **Composable**: Use Stillwater's `and`, `or`, `not` combinators
- **Type Safe**: Leverage Stillwater's predicate types
- **Zero-Cost**: Predicates compile to efficient code

## Acceptance Criteria

- [ ] `EnforcementRules` struct with max_attempts, timeout, custom checks
- [ ] Uses `Validation<(), NonEmptyVec<ViolationError>>` for accumulation
- [ ] Integrates Stillwater predicate combinators
- [ ] `enforce()` returns ALL violations at once
- [ ] Predicates for common checks (timeout, attempts, custom)
- [ ] `ViolationStrategy` enum (Abort, Retry, IgnoreAndLog)
- [ ] Builder API for creating enforcement rules
- [ ] Transitions can have optional enforcement rules
- [ ] All tests demonstrate error accumulation
- [ ] Documentation shows predicate composition patterns

## Technical Details

### Implementation Approach

Use Stillwater's Validation for fail-completely semantics and predicate combinators for composable checks.

### Violation Errors

```rust
use chrono::{DateTime, Utc};
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur when enforcing transition policies
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ViolationError {
    #[error("Maximum attempts ({max}) exceeded (current: {current})")]
    MaxAttemptsExceeded { max: usize, current: usize },

    #[error("Timeout ({timeout:?}) exceeded (elapsed: {elapsed:?})")]
    TimeoutExceeded { timeout: Duration, elapsed: Duration },

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
```

### Enforcement Context

```rust
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
```

### Enforcement Rules with Validation

```rust
use stillwater::prelude::*;
use stillwater::validation::predicate::*;
use crate::core::State;

/// Enforcement rules for state transitions.
/// Uses Validation to accumulate ALL violations.
pub struct EnforcementRules<S: State> {
    max_attempts: Option<usize>,
    timeout: Option<Duration>,
    required_checks: Vec<Box<dyn Fn(&TransitionContext<S>) -> Validation<(), ViolationError> + Send + Sync>>,
    on_violation: ViolationStrategy,
}

impl<S: State> EnforcementRules<S> {
    /// Enforce all rules, accumulating ALL violations.
    /// Returns Validation::Success(()) if all checks pass.
    /// Returns Validation::Failure with ALL violations if any fail.
    pub fn enforce(&self, context: &TransitionContext<S>) -> Validation<(), NonEmptyVec<ViolationError>> {
        let mut checks: Vec<Validation<(), ViolationError>> = Vec::new();

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
                Validation::fail(ViolationError::TimeoutExceeded {
                    timeout,
                    elapsed,
                })
            } else {
                Validation::success(())
            };
            checks.push(check);
        }

        // Run custom checks
        for check_fn in &self.required_checks {
            checks.push(check_fn(context));
        }

        // Accumulate ALL failures
        Validation::sequence(checks).map(|_| ())
    }

    pub fn violation_strategy(&self) -> ViolationStrategy {
        self.on_violation
    }
}
```

### Builder API

```rust
use std::time::Duration;

/// Builder for creating enforcement rules
pub struct EnforcementBuilder<S: State> {
    max_attempts: Option<usize>,
    timeout: Option<Duration>,
    required_checks: Vec<Box<dyn Fn(&TransitionContext<S>) -> Validation<(), ViolationError> + Send + Sync>>,
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
        F: Fn(&TransitionContext<S>) -> Validation<(), ViolationError> + Send + Sync + 'static,
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
```

### Integration with Transitions

```rust
// Update Transition from Spec 002 to include enforcement
pub struct Transition<S: State, Env> {
    pub from: S,
    pub to: S,
    pub guard: Option<Guard<S>>,
    pub action: BoxedEffect<TransitionResult<S>, TransitionError, Env>,
    pub enforcement: Option<EnforcementRules<S>>, // NEW
}
```

### Module Structure

```
mindset/
├── src/
│   ├── enforcement/
│   │   ├── mod.rs
│   │   ├── rules.rs         # EnforcementRules
│   │   ├── builder.rs       # EnforcementBuilder
│   │   ├── violations.rs    # ViolationError, Strategy
│   │   └── context.rs       # TransitionContext
```

## Dependencies

- **Prerequisites**: Specification 001, Specification 002
- **Affected Components**: Transition (add enforcement field)
- **External Dependencies**:
  - `stillwater = { version = "0.11", features = ["async"] }`

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn enforcement_accumulates_all_violations() {
        let rules = EnforcementBuilder::new()
            .max_attempts(3)
            .timeout(Duration::from_secs(5))
            .require_pred(
                |_ctx| false,
                "Custom check always fails".to_string()
            )
            .build();

        let context = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 5,  // Exceeds max
            started_at: Utc::now() - chrono::Duration::seconds(10), // Exceeds timeout
        };

        let result = rules.enforce(&context);

        // Should have ALL THREE violations
        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 3);

                // Check we got all expected errors
                let error_types: Vec<_> = errors.iter()
                    .map(|e| std::mem::discriminant(e))
                    .collect();

                assert!(error_types.iter().any(|e|
                    matches!(e, std::mem::discriminant(&ViolationError::MaxAttemptsExceeded { max: 0, current: 0 }))
                ));
                assert!(error_types.iter().any(|e|
                    matches!(e, std::mem::discriminant(&ViolationError::TimeoutExceeded {
                        timeout: Duration::ZERO,
                        elapsed: Duration::ZERO
                    }))
                ));
                assert!(error_types.iter().any(|e|
                    matches!(e, std::mem::discriminant(&ViolationError::CustomCheckFailed { message: String::new() }))
                ));
            }
            Validation::Success(_) => panic!("Expected failures, got success"),
        }
    }

    #[test]
    fn enforcement_succeeds_when_all_checks_pass() {
        let rules = EnforcementBuilder::new()
            .max_attempts(10)
            .timeout(Duration::from_secs(60))
            .require_pred(
                |_ctx| true,
                "This check always passes".to_string()
            )
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
}
```

### Integration Tests

```rust
#[tokio::test]
async fn transition_with_enforcement_blocks_on_violation() {
    let mut machine = StateMachine::new(TestState::Initial);

    let enforcement = EnforcementBuilder::new()
        .max_attempts(2)
        .on_violation(ViolationStrategy::Abort)
        .build();

    let transition = Transition {
        from: TestState::Initial,
        to: TestState::Processing,
        guard: None,
        action: pure(TransitionResult::Retry {
            feedback: "Retrying...".to_string(),
            current_state: TestState::Initial,
        }).boxed(),
        enforcement: Some(enforcement),
    };

    machine.add_transition(transition);

    let env = TestEnv { should_succeed: false };

    // First attempt - should succeed
    let result1 = machine.step().run(&env).await;
    assert!(result1.is_ok());

    // Second attempt - should succeed
    let result2 = machine.step().run(&env).await;
    assert!(result2.is_ok());

    // Third attempt - should fail (max attempts exceeded)
    let result3 = machine.step().run(&env).await;
    assert!(result3.is_err());
}
```

## Documentation Requirements

### Code Documentation

- Explain Validation-based error accumulation
- Show predicate composition examples
- Document violation strategy patterns

### User Documentation

Create `docs/enforcement.md`:
- Why error accumulation matters
- Stillwater Validation integration
- Common enforcement patterns
- Custom check examples

### Architecture Updates

Update README.md:
- Enforcement system overview
- Validation vs Result semantics
- Benefits of fail-completely approach

## Implementation Notes

### Why Validation Over Result

**Result (fail-fast)**:
```rust
// Returns FIRST error only
fn check(ctx: &Context) -> Result<(), Error> {
    check_attempts(ctx)?;  // Stops here if fails
    check_timeout(ctx)?;   // Never reached
    check_custom(ctx)?;    // Never reached
    Ok(())
}
```

**Validation (fail-completely)**:
```rust
// Returns ALL errors
fn enforce(ctx: &Context) -> Validation<(), NonEmptyVec<Error>> {
    Validation::sequence(vec![
        check_attempts(ctx),   // Always runs
        check_timeout(ctx),    // Always runs
        check_custom(ctx),     // Always runs
    ])
}
```

### Predicate Combinators

Future enhancement could use Stillwater predicates:
```rust
use stillwater::validation::predicate::*;

let timeout_pred = Predicate::new(|ctx: &TransitionContext| {
    ctx.elapsed() <= timeout
});

let attempts_pred = Predicate::new(|ctx: &TransitionContext| {
    ctx.attempt <= max_attempts
});

let combined = timeout_pred.and(attempts_pred);
```

### Performance

- Validation accumulation has minimal overhead
- All checks run regardless of failures
- This is acceptable for policy enforcement use case
- User gets comprehensive feedback in one attempt

## Migration and Compatibility

No breaking changes to existing code. Enforcement is optional:
- Transitions without enforcement work as before
- Transitions with enforcement validate before execution
- Existing tests unaffected
