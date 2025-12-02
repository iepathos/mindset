# Enforcement System

A validation-based policy enforcement system for state transitions that collects ALL violations instead of failing fast.

## Table of Contents

- [Why Error Accumulation Matters](#why-error-accumulation-matters)
- [Validation vs Result](#validation-vs-result)
- [Stillwater Integration](#stillwater-integration)
- [Common Enforcement Patterns](#common-enforcement-patterns)
- [Custom Check Examples](#custom-check-examples)
- [Violation Strategies](#violation-strategies)
- [Best Practices](#best-practices)

## Why Error Accumulation Matters

Traditional validation with `Result` follows a fail-fast approach: it stops at the first error encountered. While this can be efficient for internal error handling, it creates a poor user experience when validating complex inputs or policies.

Consider a state transition with three enforcement rules:
1. Maximum retry attempts not exceeded
2. Timeout not exceeded
3. Custom business logic satisfied

**With Result (fail-fast)**:
```rust
// User fixes max attempts violation
machine.transition(state)?;  // Error: Max attempts exceeded

// After fixing, they discover timeout violation
machine.transition(state)?;  // Error: Timeout exceeded

// After fixing, they discover custom check violation
machine.transition(state)?;  // Error: Custom check failed
```

The user must fix errors one at a time, discovering new violations only after fixing previous ones. This requires three attempts to discover all three violations.

**With Validation (accumulate-all)**:
```rust
// User sees ALL violations at once
machine.transition(state);
// Error:
//   - Max attempts exceeded (3 max, got 5)
//   - Timeout exceeded (30s max, elapsed 45s)
//   - Custom check failed: Resource unavailable
```

The user sees all violations in a single attempt and can fix everything at once. This is the core philosophy of Stillwater's `Validation` type.

## Validation vs Result

### Result Type
- **Philosophy**: Fail fast, early return
- **Use case**: Internal error propagation where you want to stop immediately
- **Error handling**: `?` operator for early return
- **Collection**: Cannot accumulate multiple errors natively

```rust
fn validate_fast(ctx: &Context) -> Result<(), String> {
    if ctx.attempt > max {
        return Err("Max attempts exceeded");  // Stops here
    }
    if ctx.elapsed > timeout {
        return Err("Timeout exceeded");  // Never reached if first check fails
    }
    Ok(())
}
```

### Validation Type
- **Philosophy**: Collect all errors, fail completely
- **Use case**: User-facing validation, policy enforcement
- **Error handling**: Accumulation combinators
- **Collection**: Native support for gathering all errors

```rust
fn validate_all(ctx: &Context) -> Validation<(), NonEmptyVec<ViolationError>> {
    let checks = vec![
        check_attempts(ctx),  // Runs even if previous fails
        check_timeout(ctx),   // Runs even if previous fails
        check_custom(ctx),    // Runs even if previous fails
    ];

    Validation::all_vec(checks).map(|_| ())
    // Returns ALL failures if any exist
}
```

### Key Differences

| Aspect | Result | Validation |
|--------|--------|------------|
| Error count | Single error | Multiple errors |
| Execution | Short-circuits | Runs all checks |
| Composition | `?` operator | Combinators (`all`, `map`, etc.) |
| User feedback | One error at a time | All errors at once |
| Best for | Internal logic | User-facing validation |

## Stillwater Integration

Mindset's enforcement system is built on Stillwater's `Validation` type, which provides:

### Core Type

```rust
pub enum Validation<T, E> {
    Success(T),
    Failure(E),  // E is typically NonEmptyVec<Error>
}
```

### Key Combinators

#### `all_vec` - Accumulate results
```rust
let checks: Vec<Validation<(), NonEmptyVec<Error>>> = vec![
    check1(),
    check2(),
    check3(),
];

// Runs all checks and accumulates failures
let result = Validation::all_vec(checks);
// Returns Failure with ALL errors if any fail
```

#### `map` - Transform success value
```rust
Validation::success(5)
    .map(|x| x * 2)  // Success(10)
```

#### `and_then` - Chain validations
```rust
validate_input()
    .and_then(|x| validate_business_logic(x))
    .and_then(|x| validate_constraints(x))
```

### NonEmptyVec

Stillwater guarantees at least one error on failure using `NonEmptyVec`:

```rust
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    pub fn len(&self) -> usize { 1 + self.tail.len() }
    pub fn iter(&self) -> impl Iterator<Item = &T>
    // ... other methods
}
```

This ensures that `Validation::Failure` always contains at least one error, eliminating the "empty error list" edge case.

## Common Enforcement Patterns

### Pattern 1: Built-in Checks

Maximum attempts and timeout enforcement:

```rust
use mindset::enforcement::{EnforcementBuilder, ViolationStrategy};
use std::time::Duration;

let rules = EnforcementBuilder::new()
    .max_attempts(3)
    .timeout(Duration::from_secs(30))
    .on_violation(ViolationStrategy::Abort)
    .build();
```

### Pattern 2: Simple Predicate Check

Boolean condition with error message:

```rust
let rules = EnforcementBuilder::new()
    .require_pred(
        |ctx| ctx.from.is_ready(),
        "Source state must be ready".to_string()
    )
    .build();
```

### Pattern 3: Complex Validation

Full `Validation` return for complex logic:

```rust
let rules = EnforcementBuilder::new()
    .require(|ctx: &TransitionContext<MyState>| {
        let mut errors = Vec::new();

        if !ctx.from.has_permission() {
            errors.push(ViolationError::CustomCheckFailed {
                message: "Missing permission".to_string(),
            });
        }

        if !ctx.to.is_valid_target() {
            errors.push(ViolationError::CustomCheckFailed {
                message: "Invalid target state".to_string(),
            });
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::fail_many(errors)
        }
    })
    .build();
```

### Pattern 4: Combining Multiple Rules

Chain multiple checks:

```rust
let rules = EnforcementBuilder::new()
    .max_attempts(5)
    .timeout(Duration::from_secs(60))
    .require_pred(
        |ctx| ctx.from.is_initialized(),
        "State must be initialized".to_string()
    )
    .require_pred(
        |ctx| !ctx.to.is_locked(),
        "Target state is locked".to_string()
    )
    .require(|ctx| check_business_logic(ctx))
    .on_violation(ViolationStrategy::Retry)
    .build();
```

All checks run and all violations are collected.

## Custom Check Examples

### Example 1: Resource Availability

Check if required resources are available:

```rust
fn check_resources<S: State>(
    ctx: &TransitionContext<S>
) -> Validation<(), NonEmptyVec<ViolationError>> {
    let available = get_available_resources();
    let required = ctx.to.required_resources();

    let missing: Vec<_> = required.iter()
        .filter(|r| !available.contains(r))
        .collect();

    if missing.is_empty() {
        Validation::success(())
    } else {
        Validation::fail(ViolationError::CustomCheckFailed {
            message: format!("Missing resources: {:?}", missing),
        })
    }
}

let rules = EnforcementBuilder::new()
    .require(check_resources)
    .build();
```

### Example 2: Time Window Enforcement

Ensure transitions only happen during allowed time windows:

```rust
use chrono::{NaiveTime, Utc};

fn check_time_window<S: State>(
    ctx: &TransitionContext<S>
) -> Validation<(), NonEmptyVec<ViolationError>> {
    let now = Utc::now().time();
    let start = NaiveTime::from_hms(9, 0, 0);
    let end = NaiveTime::from_hms(17, 0, 0);

    if now >= start && now <= end {
        Validation::success(())
    } else {
        Validation::fail(ViolationError::CustomCheckFailed {
            message: format!(
                "Transitions only allowed between {}:00 and {}:00",
                start.hour(), end.hour()
            ),
        })
    }
}
```

### Example 3: Dependency Validation

Verify dependent states before transition:

```rust
fn check_dependencies<S: State>(
    ctx: &TransitionContext<S>
) -> Validation<(), NonEmptyVec<ViolationError>> {
    let dependencies = ctx.to.get_dependencies();
    let mut errors = Vec::new();

    for dep in dependencies {
        if !dep.is_satisfied() {
            errors.push(ViolationError::CustomCheckFailed {
                message: format!("Dependency not satisfied: {}", dep.name()),
            });
        }
    }

    if errors.is_empty() {
        Validation::success(())
    } else {
        Validation::fail_many(errors)
    }
}
```

### Example 4: Rate Limiting

Prevent too many transitions in a time period:

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct RateLimiter {
    transitions: Arc<Mutex<HashMap<String, Vec<DateTime<Utc>>>>>,
    max_per_minute: usize,
}

impl RateLimiter {
    fn check<S: State>(
        &self,
        ctx: &TransitionContext<S>
    ) -> Validation<(), NonEmptyVec<ViolationError>> {
        let key = format!("{} -> {}", ctx.from.name(), ctx.to.name());
        let mut map = self.transitions.lock().unwrap();

        let now = Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);

        // Clean old entries and count recent
        let recent = map.entry(key.clone())
            .or_insert_with(Vec::new)
            .drain_filter(|t| *t > one_minute_ago)
            .collect::<Vec<_>>();

        if recent.len() >= self.max_per_minute {
            Validation::fail(ViolationError::CustomCheckFailed {
                message: format!(
                    "Rate limit exceeded: {} transitions in last minute",
                    recent.len()
                ),
            })
        } else {
            map.insert(key, recent);
            Validation::success(())
        }
    }
}
```

## Violation Strategies

When enforcement rules are violated, you can control the behavior:

### Abort (Default)

Stop transition permanently:

```rust
let rules = EnforcementBuilder::new()
    .max_attempts(3)
    .on_violation(ViolationStrategy::Abort)
    .build();

// Transition fails permanently when attempts exceeded
```

Use when:
- Violations indicate unrecoverable errors
- Continuing would violate business invariants
- Safety-critical transitions

### Retry

Allow retry despite violations:

```rust
let rules = EnforcementBuilder::new()
    .timeout(Duration::from_secs(30))
    .on_violation(ViolationStrategy::Retry)
    .build();

// Timeout violations allow retry with exponential backoff
```

Use when:
- Violations may be temporary (timeouts, resource unavailability)
- Retry with backoff might succeed
- User intervention can fix the issue

### IgnoreAndLog

Continue but log warning:

```rust
let rules = EnforcementBuilder::new()
    .require_pred(|ctx| ctx.from.is_optimal(), "Non-optimal".to_string())
    .on_violation(ViolationStrategy::IgnoreAndLog)
    .build();

// Non-optimal state logs warning but transition proceeds
```

Use when:
- Violations are warnings, not errors
- Transition can safely proceed with degraded state
- Monitoring and logging is sufficient

## Best Practices

### 1. Design for Complete Feedback

Write checks that can all run independently:

```rust
// GOOD: Independent checks
let rules = EnforcementBuilder::new()
    .require_pred(|ctx| check_a(ctx), "A failed".to_string())
    .require_pred(|ctx| check_b(ctx), "B failed".to_string())
    .require_pred(|ctx| check_c(ctx), "C failed".to_string())
    .build();

// BAD: Dependent checks (defeats accumulation purpose)
let rules = EnforcementBuilder::new()
    .require(|ctx| {
        check_a(ctx).and_then(|_|
            check_b(ctx).and_then(|_|
                check_c(ctx)))
    })
    .build();
```

### 2. Provide Actionable Error Messages

Make messages specific and actionable:

```rust
// GOOD: Specific and actionable
ViolationError::CustomCheckFailed {
    message: "Order total ($0.00) must be greater than $0.00".to_string(),
}

// BAD: Vague
ViolationError::CustomCheckFailed {
    message: "Invalid order".to_string(),
}
```

### 3. Use Appropriate Violation Strategies

Match strategy to the nature of violations:

```rust
let rules = EnforcementBuilder::new()
    // Critical safety check - abort
    .require_pred(
        |ctx| ctx.from.is_safe(),
        "Safety violation".to_string()
    )
    .on_violation(ViolationStrategy::Abort)
    .build();

let rules = EnforcementBuilder::new()
    // Temporary resource issue - retry
    .require_pred(
        |ctx| resources_available(),
        "Resources unavailable".to_string()
    )
    .on_violation(ViolationStrategy::Retry)
    .build();

let rules = EnforcementBuilder::new()
    // Performance optimization - log
    .require_pred(
        |ctx| ctx.from.is_optimized(),
        "Not optimized".to_string()
    )
    .on_violation(ViolationStrategy::IgnoreAndLog)
    .build();
```

### 4. Keep Checks Pure and Fast

Enforcement checks should be pure functions without side effects:

```rust
// GOOD: Pure check
fn check_balance(ctx: &TransitionContext<Account>) -> bool {
    ctx.from.balance >= 0.0
}

// BAD: Side effect in check
fn check_balance_bad(ctx: &TransitionContext<Account>) -> bool {
    log::info!("Checking balance");  // Side effect!
    database::update_last_check(ctx.from.id);  // Side effect!
    ctx.from.balance >= 0.0
}
```

### 5. Compose Checks for Reusability

Extract common checks for reuse:

```rust
fn check_initialized<S: State>(
    ctx: &TransitionContext<S>
) -> Validation<(), NonEmptyVec<ViolationError>> {
    if ctx.from.is_initialized() {
        Validation::success(())
    } else {
        Validation::fail(ViolationError::CustomCheckFailed {
            message: "State not initialized".to_string(),
        })
    }
}

fn check_not_locked<S: State>(
    ctx: &TransitionContext<S>
) -> Validation<(), NonEmptyVec<ViolationError>> {
    if !ctx.to.is_locked() {
        Validation::success(())
    } else {
        Validation::fail(ViolationError::CustomCheckFailed {
            message: "Target state is locked".to_string(),
        })
    }
}

// Reuse in multiple rule sets
let rules = EnforcementBuilder::new()
    .require(check_initialized)
    .require(check_not_locked)
    .build();
```

### 6. Test All Violation Scenarios

Write tests that verify all violations are collected:

```rust
#[test]
fn test_multiple_violations_collected() {
    let rules = EnforcementBuilder::new()
        .max_attempts(3)
        .timeout(Duration::from_secs(5))
        .require_pred(|_| false, "Custom check failed".to_string())
        .build();

    let ctx = TransitionContext {
        from: State::A,
        to: State::B,
        attempt: 5,
        started_at: Utc::now() - chrono::Duration::seconds(10),
    };

    let result = rules.enforce(&ctx);

    if let Validation::Failure(errors) = result {
        assert_eq!(errors.len(), 3);  // All three violations collected

        assert!(errors.iter().any(|e|
            matches!(e, ViolationError::MaxAttemptsExceeded { .. })));
        assert!(errors.iter().any(|e|
            matches!(e, ViolationError::TimeoutExceeded { .. })));
        assert!(errors.iter().any(|e|
            matches!(e, ViolationError::CustomCheckFailed { .. })));
    } else {
        panic!("Expected failures");
    }
}
```

## Summary

The enforcement system provides:

- **Complete feedback**: Users see all violations at once via `Validation`
- **Stillwater integration**: Built on solid functional foundations
- **Flexible policies**: Combine built-in and custom checks
- **Configurable responses**: Abort, retry, or log violations
- **Type safety**: Compile-time guarantees through strong typing

By using `Validation` instead of `Result`, the enforcement system prioritizes developer experience and enables comprehensive policy checking in a single evaluation.
