---
number: 002
title: Documentation Improvements and Cloneable Enforcement
category: compatibility
priority: high
status: draft
dependencies: [001]
created: 2025-12-01
---

# Specification 002: Documentation Improvements and Cloneable Enforcement

**Category**: compatibility
**Priority**: high
**Status**: draft
**Dependencies**: [001 - Enforcement Integration]

## Context

The mindset library has several documentation and usability gaps:

1. **Missing patterns.md Documentation**: The README references `docs/patterns.md` at line 614 in the "Further Reading" section, but this file does not exist. This creates a broken documentation link.

2. **Enforcement Not Cloneable**: Transitions with enforcement rules cannot be cloned due to `Box<dyn Fn>` in custom predicates. The `Transition::clone()` implementation explicitly sets `enforcement: None` (src/effects/transition.rs:71), which silently drops enforcement rules during cloning. This limits use cases where transitions need to be cloned (e.g., dynamic machine construction, transition templates).

3. **Incomplete Example Coverage**: The validation_enforcement example (after Spec 001) will demonstrate enforcement execution, but there's no example showing the step/apply pattern with enforcement in realistic scenarios.

These issues impact developer experience and limit the flexibility of the enforcement system.

## Objective

1. Create comprehensive state machine patterns documentation
2. Make enforcement rules cloneable by wrapping checks in Arc
3. Add example demonstrating step/apply pattern with enforcement
4. Document enforcement integration patterns in effects guide

## Requirements

### Functional Requirements

- FR1: Create `docs/patterns.md` covering common state machine patterns
- FR2: Modify `EnforcementRules` to use `Arc<dyn Fn>` instead of `Box<dyn Fn>`
- FR3: Implement proper `Clone` for `EnforcementRules`
- FR4: Verify `Transition::clone()` preserves enforcement rules
- FR5: Create example showing step/apply with enforcement (e.g., workflow with retries)
- FR6: Update `docs/effects-guide.md` with enforcement integration patterns
- FR7: Add cross-references between documentation files

### Non-Functional Requirements

- NFR1: patterns.md must be comprehensive (500+ lines)
- NFR2: Arc wrapping must not impact performance (<5% overhead)
- NFR3: Documentation must include runnable code examples
- NFR4: Example must be realistic (not toy code)
- NFR5: All documentation must follow project style guide

## Acceptance Criteria

- [ ] `docs/patterns.md` exists and covers 8+ common patterns
- [ ] patterns.md includes: guards, actions, composition, error handling, testing
- [ ] `EnforcementRules` uses `Arc<dyn Fn>` for all check functions
- [ ] `EnforcementRules` implements `Clone` trait
- [ ] `Transition::clone()` preserves enforcement rules correctly
- [ ] Tests verify cloned transitions maintain enforcement behavior
- [ ] New example: `examples/workflow_with_retries.rs`
- [ ] Example shows step/apply pattern with multiple retry attempts
- [ ] Example demonstrates all three violation strategies
- [ ] `docs/effects-guide.md` updated with enforcement sections
- [ ] Effects guide shows enforcement in effectful transitions
- [ ] All docs have proper cross-references and links
- [ ] Documentation builds without warnings
- [ ] cargo doc generates correct links

## Technical Details

### Implementation Approach

#### 1. Make Enforcement Cloneable

**Current Implementation** (src/enforcement/rules.rs:10-21):
```rust
pub type ValidationCheck<S> =
    Box<dyn Fn(&TransitionContext<S>) -> Validation<(), NonEmptyVec<ViolationError>> + Send + Sync>;

pub struct EnforcementRules<S: State> {
    pub(crate) max_attempts: Option<usize>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) required_checks: Vec<ValidationCheck<S>>,  // Not cloneable
    pub(crate) on_violation: ViolationStrategy,
}
```

**Updated Implementation**:
```rust
use std::sync::Arc;

pub type ValidationCheck<S> =
    Arc<dyn Fn(&TransitionContext<S>) -> Validation<(), NonEmptyVec<ViolationError>> + Send + Sync>;

pub struct EnforcementRules<S: State> {
    pub(crate) max_attempts: Option<usize>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) required_checks: Vec<ValidationCheck<S>>,  // Now cloneable via Arc::clone
    pub(crate) on_violation: ViolationStrategy,
}

impl<S: State> Clone for EnforcementRules<S> {
    fn clone(&self) -> Self {
        Self {
            max_attempts: self.max_attempts,
            timeout: self.timeout,
            required_checks: self.required_checks.clone(),  // Arc::clone for each
            on_violation: self.on_violation,
        }
    }
}
```

**Update EnforcementBuilder** (src/enforcement/builder.rs):
```rust
pub fn require_pred<F>(mut self, predicate: F, message: String) -> Self
where
    F: Fn(&TransitionContext<S>) -> bool + Send + Sync + 'static,
{
    let check = Arc::new(move |ctx: &TransitionContext<S>| {  // Arc instead of Box
        if predicate(ctx) {
            Validation::success(())
        } else {
            Validation::fail(ViolationError::CustomCheckFailed {
                message: message.clone(),
            })
        }
    });

    self.required_checks.push(check);
    self
}
```

**Update Transition Clone** (src/effects/transition.rs):
```rust
impl<S: State, Env> Clone for Transition<S, Env> {
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            guard: self.guard.clone(),
            action: Arc::clone(&self.action),
            enforcement: self.enforcement.clone(),  // Now works!
        }
    }
}
```

#### 2. Create patterns.md Documentation

Structure:
```markdown
# State Machine Patterns

## Table of Contents
1. Basic Patterns
2. Guard Patterns
3. Action Patterns
4. Composition Patterns
5. Error Handling Patterns
6. Testing Patterns
7. Performance Patterns
8. Advanced Patterns

## Basic Patterns

### Simple State Transition
[Code example]

### Cyclic State Machine
[Code example]

### Linear Workflow
[Code example]

## Guard Patterns

### State-Based Guards
### Time-Based Guards
### Resource-Based Guards
### Composite Guards

## Action Patterns

### Pure Actions
### Effectful Actions
### Compensating Actions
### Idempotent Actions

## Composition Patterns

### Parallel Transitions
### Sequential Workflows
### Conditional Branching
### State Aggregation

## Error Handling Patterns

### Retry with Backoff
### Circuit Breaker
### Fallback State
### Error State Transitions

## Testing Patterns

### Mock Environments
### Property-Based Testing
### Scenario Testing
### Time-Travel Testing

## Performance Patterns

### Lazy Evaluation
### Batch Transitions
### Caching Guards
### Zero-Copy State

## Advanced Patterns

### Hierarchical State Machines
### State Machine Composition
### Event Sourcing
### CQRS with State Machines
```

#### 3. Create Workflow with Retries Example

File: `examples/workflow_with_retries.rs`

```rust
//! Workflow with Retries
//!
//! Demonstrates the step/apply pattern with enforcement and retry logic.
//! Shows realistic error handling with multiple retry attempts.

use mindset::{
    builder::{StateMachineBuilder, TransitionBuilder},
    enforcement::{EnforcementBuilder, ViolationStrategy},
    state_enum,
};
use std::time::Duration;

state_enum! {
    enum JobState {
        Queued,
        Processing,
        Completed,
        Failed,
    }
    final: [Completed, Failed]
    error: [Failed]
}

struct Job {
    id: u64,
    attempts: usize,
    max_retries: usize,
}

trait JobExecutor {
    fn execute(&mut self, job: &Job) -> Result<(), String>;
    fn should_retry(&self, error: &str) -> bool;
}

struct SimulatedExecutor {
    attempt_count: usize,
    fail_until_attempt: usize,
}

impl JobExecutor for SimulatedExecutor {
    fn execute(&mut self, job: &Job) -> Result<(), String> {
        self.attempt_count += 1;
        if self.attempt_count < self.fail_until_attempt {
            Err(format!("Transient failure (attempt {})", self.attempt_count))
        } else {
            Ok(())
        }
    }

    fn should_retry(&self, error: &str) -> bool {
        error.contains("Transient")
    }
}

#[tokio::main]
async fn main() {
    println!("=== Workflow with Retries Example ===\n");

    // Create job with retry policy
    let job = Job {
        id: 42,
        attempts: 0,
        max_retries: 3,
    };

    // Build machine with enforcement
    let machine = StateMachineBuilder::new()
        .initial(JobState::Queued)
        .add_transition(
            TransitionBuilder::new()
                .from(JobState::Queued)
                .to(JobState::Processing)
                .succeeds()
                .enforce(
                    EnforcementBuilder::new()
                        .max_attempts(3)
                        .timeout(Duration::from_secs(30))
                        .on_violation(ViolationStrategy::Retry)
                        .build()
                )
                .build()
                .unwrap()
        )
        .build()
        .unwrap();

    // Execute with retries
    let mut env = SimulatedExecutor {
        attempt_count: 0,
        fail_until_attempt: 2,
    };

    loop {
        match machine.step().run(&env).await {
            Ok((from, result, attempt)) => {
                machine.apply_result(from, result.clone(), attempt);

                match result {
                    StepResult::Transitioned(state) => {
                        println!("Transitioned to {:?}", state);
                        break;
                    }
                    StepResult::Retry { feedback, attempts } => {
                        println!("Retry {} of {}: {}", attempts, job.max_retries, feedback);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    StepResult::Aborted { reason, .. } => {
                        println!("Aborted: {}", reason);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
```

#### 4. Update Effects Guide

Add to `docs/effects-guide.md` after line 100:

```markdown
## Enforcement in Effectful Transitions

### Basic Enforcement

Enforcement rules validate transition preconditions before executing actions:

```rust
let transition = TransitionBuilder::new()
    .from(State::Idle)
    .to(State::Running)
    .action(|| perform_startup())
    .enforce(
        EnforcementBuilder::new()
            .max_attempts(3)
            .timeout(Duration::from_secs(30))
            .build()
    )
    .build()
    .unwrap();
```

### Enforcement with Effects

Enforcement checks are pure, but can access the transition context:

```rust
let rules = EnforcementBuilder::new()
    .require_pred(
        |ctx| check_resource_available(&ctx.from),
        "Resource must be available".to_string()
    )
    .on_violation(ViolationStrategy::Retry)
    .build();
```

### Violation Strategies

Choose the appropriate strategy for your use case:

- **Abort**: Critical checks that must pass (e.g., safety invariants)
- **Retry**: Temporary failures that may succeed later (e.g., resource availability)
- **IgnoreAndLog**: Non-critical checks for monitoring (e.g., performance warnings)
```

### Architecture Changes

- `EnforcementRules` struct signature changes (Arc instead of Box)
- `Transition::clone()` no longer discards enforcement
- New example file in examples/
- New documentation file in docs/

### Data Structures

No new data structures, but modified ownership model:
- `ValidationCheck<S>` now uses `Arc` instead of `Box`
- Enables `Clone` implementation for `EnforcementRules`
- Minimal runtime overhead (pointer indirection already exists)

### APIs and Interfaces

**Breaking Change**: `EnforcementBuilder::require_pred` still has same signature, but internal implementation uses Arc. This is transparent to users.

**New API**: `EnforcementRules::clone()` now available

## Dependencies

- **Prerequisites**:
  - [001 - Enforcement Integration] - Enforcement must actually work before documenting patterns
- **Affected Components**:
  - `src/enforcement/rules.rs` - Change Box to Arc
  - `src/enforcement/builder.rs` - Update Arc construction
  - `src/effects/transition.rs` - Fix clone implementation
  - `docs/patterns.md` - New file
  - `docs/effects-guide.md` - Additions
  - `examples/workflow_with_retries.rs` - New file
- **External Dependencies**: None (uses existing std::sync::Arc)

## Testing Strategy

### Unit Tests

Add to `src/enforcement/rules.rs::tests`:

```rust
#[test]
fn enforcement_rules_are_cloneable() {
    let rules = EnforcementBuilder::new()
        .max_attempts(3)
        .timeout(Duration::from_secs(10))
        .require_pred(|_| true, "test".to_string())
        .build();

    let cloned = rules.clone();

    // Verify clone has same configuration
    assert_eq!(cloned.max_attempts, rules.max_attempts);
    assert_eq!(cloned.timeout, rules.timeout);
    assert_eq!(cloned.required_checks.len(), rules.required_checks.len());
}

#[test]
fn cloned_enforcement_enforces_correctly() {
    let rules = EnforcementBuilder::new()
        .max_attempts(2)
        .build();

    let cloned = rules.clone();

    let ctx = TransitionContext {
        from: TestState::Initial,
        to: TestState::Processing,
        attempt: 5,
        started_at: Utc::now(),
    };

    // Both original and clone should enforce
    let result1 = rules.enforce(&ctx);
    let result2 = cloned.enforce(&ctx);

    assert!(matches!(result1, Validation::Failure(_)));
    assert!(matches!(result2, Validation::Failure(_)));
}
```

Add to `src/effects/transition.rs::tests`:

```rust
#[test]
fn transition_clone_preserves_enforcement() {
    let rules = EnforcementBuilder::new()
        .max_attempts(3)
        .build();

    let transition = Transition {
        from: TestState::Start,
        to: TestState::Middle,
        guard: None,
        action: Arc::new(|| pure(TransitionResult::Success(TestState::Middle)).boxed()),
        enforcement: Some(rules),
    };

    let cloned = transition.clone();

    assert!(cloned.enforcement.is_some());
    // Verify enforcement works on clone
    let ctx = TransitionContext {
        from: TestState::Start,
        to: TestState::Middle,
        attempt: 5,
        started_at: Utc::now(),
    };

    let result = cloned.enforcement.unwrap().enforce(&ctx);
    assert!(matches!(result, Validation::Failure(_)));
}
```

### Integration Tests

Add new file `tests/enforcement_clone.rs`:

```rust
#[tokio::test]
async fn cloned_machine_preserves_enforcement() {
    // Create machine with enforced transitions
    let machine = create_machine_with_enforcement();

    // Clone transitions
    let cloned_transitions = machine.transitions.clone();

    // Build new machine with cloned transitions
    let machine2 = StateMachine::from_checkpoint(
        checkpoint,
        cloned_transitions
    ).unwrap();

    // Verify enforcement still works
    // Execute steps and verify violations are caught
}
```

### Example Tests

The new example must run successfully:
```bash
cargo run --example workflow_with_retries
# Should show retry attempts and eventual success
```

### Documentation Tests

All code examples in patterns.md must compile:
```bash
cargo test --doc
```

## Documentation Requirements

### Code Documentation

- Rustdoc for `EnforcementRules::clone()`
- Update `ValidationCheck` type alias documentation
- Add examples to `require_pred` showing Arc usage
- Document Arc overhead in performance section

### User Documentation

**Create docs/patterns.md** (500+ lines):
- Introduction to state machine patterns
- 8+ pattern categories with examples
- Cross-references to other docs
- Performance considerations for each pattern
- Testing strategies for each pattern

**Update docs/effects-guide.md**:
- Add "Enforcement in Effectful Transitions" section
- Show enforcement with step/apply pattern
- Document violation strategies in context
- Add realistic examples with retry logic

**Update examples/README.md**:
- Add entry for workflow_with_retries
- Categorize as "Advanced Patterns"
- Note that it demonstrates step/apply with enforcement

### Architecture Updates

Add to README.md "Further Reading" section:
```markdown
## Further Reading

- [Stillwater 0.11.0 Documentation](https://docs.rs/stillwater)
- [Effects Guide](docs/effects-guide.md)
- [State Machine Patterns](docs/patterns.md) ← Now exists!
- [Enforcement Guide](docs/enforcement.md)
```

## Implementation Notes

### Arc vs Box Performance

Arc has minimal overhead compared to Box:
- Both involve one heap allocation
- Arc adds atomic reference counting (typically 8-16 bytes)
- Read operations have same cost (pointer dereference)
- Clone is cheap (atomic increment)
- Drop is slightly more expensive (atomic decrement + conditional free)

For enforcement checks (executed once per transition), this overhead is negligible.

### Benchmark Considerations

Add benchmark to verify no performance regression:

```rust
#[bench]
fn bench_enforcement_with_arc(b: &mut Bencher) {
    let rules = EnforcementBuilder::new()
        .max_attempts(10)
        .require_pred(|_| true, "test".to_string())
        .build();

    let ctx = create_test_context();

    b.iter(|| {
        rules.enforce(&ctx)
    });
}
```

Target: <5% overhead vs Box (likely <1% in practice).

### Documentation Style

All docs must follow project conventions:
- Use `///` for rustdoc comments
- Include `# Example` sections
- Show both success and error cases
- Link to related types and functions
- Keep examples concise but realistic

### Cross-Reference Strategy

patterns.md should link to:
- effects-guide.md for effectful patterns
- enforcement.md for validation patterns
- builder-guide.md for construction patterns
- examples/ for full runnable code

Each pattern should have:
1. Description
2. When to use
3. Implementation code
4. Testing strategy
5. Related patterns
6. Link to full example

## Migration and Compatibility

### Breaking Changes

**Potential Issue**: Existing code using `EnforcementRules` directly (unlikely, as it's typically built via `EnforcementBuilder`).

If users were constructing `EnforcementRules` manually:
```rust
// Old (rare, not recommended):
let rules = EnforcementRules {
    required_checks: vec![Box::new(|_| Validation::success(()))],
    // ...
};

// New:
let rules = EnforcementRules {
    required_checks: vec![Arc::new(|_| Validation::success(()))],
    // ...
};
```

### API Compatibility

`EnforcementBuilder` API unchanged - users won't notice the Arc change.

Transition cloning now preserves enforcement (previously silently dropped).

### Behavior Changes

**Before**: `transition.clone()` lost enforcement rules
**After**: `transition.clone()` preserves enforcement rules

This is a fix, not a breaking change.

### Migration Path

1. Upgrade mindset to new version
2. Review any code that clones transitions
3. Verify enforcement is preserved as expected
4. No code changes needed

### Upgrade Notes

```markdown
## [0.2.0] - 2025-12-XX

### Added
- Documentation: Comprehensive state machine patterns guide (docs/patterns.md)
- Example: workflow_with_retries showing step/apply with enforcement
- Documentation: Enforcement integration patterns in effects guide

### Fixed
- Enforcement rules are now properly cloneable
- Transition cloning now preserves enforcement rules (previously silently dropped)

### Changed
- Internal: EnforcementRules now uses Arc instead of Box for checks (no API changes)
```

## Success Metrics

- [ ] All 14 acceptance criteria met
- [ ] patterns.md >500 lines with 8+ patterns
- [ ] All documentation examples compile and run
- [ ] Arc change causes <5% performance overhead
- [ ] All tests pass including new clone tests
- [ ] workflow_with_retries example demonstrates realistic retry logic
- [ ] Cross-references between docs work correctly
- [ ] cargo doc builds without warnings
