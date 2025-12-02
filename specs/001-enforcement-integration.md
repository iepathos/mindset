---
number: 001
title: Enforcement Integration in State Machine Execution
category: foundation
priority: critical
status: draft
dependencies: []
created: 2025-12-01
---

# Specification 001: Enforcement Integration in State Machine Execution

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: None

## Context

The mindset library currently implements an enforcement system based on Stillwater's `Validation` type that can accumulate all violations (max attempts, timeouts, custom predicates) and report them together. However, the enforcement rules are stored in transitions but **never actually executed** during state machine operation.

The `StateMachine::step()` method in `src/effects/machine.rs:66-116` finds applicable transitions and executes their actions, but completely ignores the `transition.enforcement` field. This means:

- Built enforcement rules have no effect on execution
- Max attempts, timeouts, and custom checks are silently ignored
- Users believe they have enforcement protection but it's not active
- The validation_enforcement example only demonstrates rule creation, not execution

This is a critical gap between advertised functionality and actual behavior that must be addressed before the library can be considered production-ready.

## Objective

Integrate enforcement rule checking into the state machine execution path so that violations are properly detected, accumulated using Validation semantics, and handled according to the configured violation strategy (Abort, Retry, IgnoreAndLog).

## Requirements

### Functional Requirements

- FR1: State machine must check enforcement rules before executing transition actions
- FR2: Must track transition start time to enable timeout checking
- FR3: Must create proper `TransitionContext` with all required fields for enforcement checks
- FR4: Must accumulate ALL violations using Stillwater's `Validation::all_vec`
- FR5: Must handle violations according to the configured `ViolationStrategy`
  - Abort: Return error effect immediately, transition to error state if defined
  - Retry: Allow transition to be retried, increment attempt counter
  - IgnoreAndLog: Log warning but proceed with transition
- FR6: Must update attempt counter properly for retry scenarios
- FR7: Must preserve history entries for aborted transitions

### Non-Functional Requirements

- NFR1: Zero-cost when enforcement is not configured (Option<EnforcementRules>)
- NFR2: Enforcement checks must be pure (no side effects)
- NFR3: Must maintain effect composition patterns (return impl Effect)
- NFR4: Must preserve existing API contracts for step() and apply_result()
- NFR5: Thread-safe enforcement rule checking

## Acceptance Criteria

- [ ] `StateMachine` tracks transition start time as `Option<DateTime<Utc>>`
- [ ] `StateMachine::step()` creates `TransitionContext` before executing actions
- [ ] `StateMachine::step()` calls `enforcement.enforce()` when rules present
- [ ] Violations with Abort strategy prevent action execution and return error
- [ ] Violations with Retry strategy allow retry with incremented attempt count
- [ ] Violations with IgnoreAndLog strategy log warning but proceed
- [ ] Timeout checks work correctly using transition start time
- [ ] Max attempts checks work correctly using attempt counter
- [ ] Custom predicate checks execute and violations are accumulated
- [ ] Multiple violations are collected and reported together (Validation semantics)
- [ ] Integration tests verify enforcement blocks invalid transitions
- [ ] Integration tests verify enforcement allows valid transitions
- [ ] Integration tests verify all violation strategies work correctly
- [ ] Example updated to show actual enforcement in action with step/apply
- [ ] No performance regression for machines without enforcement
- [ ] All existing tests continue to pass

## Technical Details

### Implementation Approach

1. **Add Transition Tracking to StateMachine**
   ```rust
   pub struct StateMachine<S: State + 'static, Env: Clone + Send + Sync + 'static> {
       // ... existing fields ...
       transition_started_at: Option<DateTime<Utc>>,
   }
   ```

2. **Modify step() to Check Enforcement**
   ```rust
   pub fn step(&mut self) -> impl Effect<...> {
       // Find transition
       let transition = self.transitions.iter().find(...)?;

       // Set start time if not set
       if self.transition_started_at.is_none() {
           self.transition_started_at = Some(Utc::now());
       }

       // Check enforcement if present
       if let Some(enforcement) = &transition.enforcement {
           let context = TransitionContext {
               from: self.current.clone(),
               to: transition.to.clone(),
               attempt: self.attempt_count,
               started_at: self.transition_started_at.unwrap(),
           };

           match enforcement.enforce(&context) {
               Validation::Failure(errors) => {
                   return self.handle_violations(errors, enforcement.violation_strategy());
               }
               Validation::Success(_) => { /* proceed */ }
           }
       }

       // Execute action
       let action = (transition.action)();
       // ... rest of existing logic
   }
   ```

3. **Add Violation Handler**
   ```rust
   fn handle_violations(
       &self,
       errors: NonEmptyVec<ViolationError>,
       strategy: ViolationStrategy,
   ) -> impl Effect<Output = (S, StepResult<S>, usize), Error = TransitionError, Env = Env> {
       match strategy {
           ViolationStrategy::Abort => {
               // Return error effect
               fail(TransitionError::EnforcementViolation {
                   violations: errors.into_vec()
               })
           }
           ViolationStrategy::Retry => {
               // Return retry result
               pure((
                   self.current.clone(),
                   StepResult::Retry {
                       feedback: format_violations(&errors),
                       attempts: self.attempt_count + 1,
                   },
                   self.attempt_count
               ))
           }
           ViolationStrategy::IgnoreAndLog => {
               eprintln!("WARNING: Enforcement violations ignored: {}", format_violations(&errors));
               // Continue with normal transition
               // (need to restructure to allow this)
           }
       }
   }
   ```

4. **Update apply_result() to Reset Timing**
   ```rust
   pub fn apply_result(&mut self, from_state: S, result: StepResult<S>, attempt_count: usize) {
       match result {
           StepResult::Transitioned(new_state) => {
               // ... existing logic ...
               self.transition_started_at = None; // Reset for next transition
           }
           StepResult::Retry { .. } => {
               self.attempt_count += 1;
               // Keep transition_started_at for timeout tracking
           }
           StepResult::Aborted { .. } => {
               // ... existing logic ...
               self.transition_started_at = None; // Reset
           }
       }
   }
   ```

### Architecture Changes

- `StateMachine` gains new field: `transition_started_at: Option<DateTime<Utc>>`
- `TransitionError` gains new variant: `EnforcementViolation { violations: Vec<ViolationError> }`
- `step()` method signature may need to become `&mut self` to update start time
  - Alternative: Return start time from step() and handle in apply_result()
- New private method: `handle_violations()` for violation strategy handling

### Data Structures

```rust
// New error variant
#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    // ... existing variants ...

    #[error("Enforcement violations: {}", format_violations(.violations))]
    EnforcementViolation {
        violations: Vec<ViolationError>
    },
}

// Helper function
fn format_violations(violations: &[ViolationError]) -> String {
    violations.iter()
        .map(|v| format!("  - {}", v))
        .collect::<Vec<_>>()
        .join("\n")
}
```

### APIs and Interfaces

No public API changes required. The enforcement integration is internal to the execution path. Users continue to:
1. Build enforcement rules with `EnforcementBuilder`
2. Attach rules to transitions with `TransitionBuilder::enforce()`
3. Execute with `step()` and `apply_result()` as before

Enforcement now actually works behind the scenes.

## Dependencies

- **Prerequisites**: None (fixes existing functionality)
- **Affected Components**:
  - `src/effects/machine.rs` - StateMachine implementation
  - `src/effects/transition.rs` - TransitionError enum
  - `src/enforcement/violations.rs` - ViolationError formatting
- **External Dependencies**: None (uses existing Stillwater and chrono)

## Testing Strategy

### Unit Tests

Add to `src/effects/machine.rs::tests`:

```rust
#[tokio::test]
async fn enforcement_blocks_transition_on_max_attempts() {
    let mut machine = create_test_machine();
    let rules = EnforcementBuilder::new()
        .max_attempts(2)
        .on_violation(ViolationStrategy::Abort)
        .build();

    // Add transition with enforcement
    machine.add_transition(create_transition_with_enforcement(rules));

    // Set attempt count to exceed max
    machine.attempt_count = 3;

    let env = TestEnv::default();
    let result = machine.step().run(&env).await;

    assert!(matches!(result, Err(TransitionError::EnforcementViolation { .. })));
}

#[tokio::test]
async fn enforcement_allows_valid_transition() {
    // Test that valid transitions proceed normally
}

#[tokio::test]
async fn enforcement_accumulates_multiple_violations() {
    // Test that max attempts + timeout + custom all reported together
}

#[tokio::test]
async fn enforcement_retry_strategy_increments_attempts() {
    // Test retry behavior
}

#[tokio::test]
async fn enforcement_timeout_checks_elapsed_time() {
    // Test timeout detection using transition_started_at
}
```

### Integration Tests

Add new file `tests/enforcement_integration.rs`:

```rust
// Full workflow tests showing enforcement in action
#[tokio::test]
async fn complete_workflow_with_enforcement() {
    // Create machine with enforced transitions
    // Execute multiple steps
    // Verify enforcement blocks/allows appropriately
    // Verify attempt counters and timing
}

#[tokio::test]
async fn enforcement_with_retries() {
    // Test retry strategy with multiple attempts
}

#[tokio::test]
async fn enforcement_violation_strategies() {
    // Test all three strategies in realistic scenarios
}
```

### Property Tests

Add to `tests/property_tests.rs`:

```rust
proptest! {
    #[test]
    fn enforcement_always_accumulates_all_violations(
        max_attempts: usize,
        timeout_secs: u64,
        attempt: usize
    ) {
        // Property: If multiple checks fail, all must be reported
    }
}
```

### Example Updates

Update `examples/validation_enforcement.rs` to show actual execution:

```rust
// Current example only shows rule creation
// New section:

println!("\n=== Example 6: Enforcement in Action ===");

// Create machine with enforced transition
let mut machine = StateMachineBuilder::new()
    .initial(TaskState::Pending)
    .add_transition(
        TransitionBuilder::new()
            .from(TaskState::Pending)
            .to(TaskState::Running)
            .succeeds()
            .enforce(
                EnforcementBuilder::new()
                    .max_attempts(3)
                    .timeout(Duration::from_secs(10))
                    .build()
            )
            .build()
            .unwrap()
    )
    .build()
    .unwrap();

// Attempt 1 - should succeed
let env = TestEnv::default();
let (from, result, attempt) = machine.step().run(&env).await.unwrap();
machine.apply_result(from, result, attempt);
println!("  Attempt 1: Success");

// Attempt 2 (simulate timeout by setting start time to past)
machine.transition_started_at = Some(Utc::now() - Duration::from_secs(15));
let result = machine.step().run(&env).await;
match result {
    Err(TransitionError::EnforcementViolation { violations }) => {
        println!("  Attempt 2: Blocked by enforcement");
        for v in violations {
            println!("    - {}", v);
        }
    }
    _ => panic!("Expected enforcement violation"),
}
```

## Documentation Requirements

### Code Documentation

- Add comprehensive rustdoc for `transition_started_at` field
- Document enforcement integration in `step()` method
- Add examples showing enforcement usage in method docs
- Document `EnforcementViolation` error variant

### User Documentation

- Update `docs/enforcement.md`:
  - Add section "How Enforcement Works" explaining integration
  - Add section "Execution Flow with Enforcement"
  - Add timing diagram showing when checks occur
- Update `docs/effects-guide.md`:
  - Add section on enforcement in effect execution
  - Show examples of enforcement with step/apply pattern
- Update `examples/README.md`:
  - Note that validation_enforcement now shows actual execution

### Architecture Updates

Add section to README.md or docs/architecture.md:

```markdown
## Enforcement Execution Flow

1. `step()` finds applicable transition
2. Sets `transition_started_at` if not already set
3. Creates `TransitionContext` with current state + timing
4. Calls `enforcement.enforce()` if rules present
5. Accumulates all violations using `Validation::all_vec`
6. Handles violations per strategy:
   - Abort: Returns error effect immediately
   - Retry: Returns retry result, keeps timing
   - IgnoreAndLog: Logs and continues
7. Executes transition action if enforcement passed
8. `apply_result()` updates state and resets timing
```

## Implementation Notes

### Ownership and Borrowing

The `step()` method currently takes `&self` and returns an effect. To set `transition_started_at`, we have two options:

**Option A**: Make `step()` take `&mut self`
- Simpler implementation
- Breaks zero-cost effect composition pattern
- Requires mutable borrow during effect execution

**Option B**: Return start time from `step()`, handle in `apply_result()`
- Preserves immutable `step()` signature
- More complex return type: `(DateTime<Utc>, StepResult, ...)`
- Caller sets timing via `apply_result()`

**Recommendation**: Option B to preserve effect semantics, but benchmark both.

### Performance Considerations

- Enforcement checks are pure (no I/O)
- `Option<EnforcementRules>` ensures zero cost when unused
- `Validation::all_vec` has O(n) complexity in number of checks
- Typical case: 2-3 checks, negligible overhead
- Consider adding `#[inline]` to enforcement path

### Thread Safety

- `EnforcementRules` contains `Box<dyn Fn>` which is `Send + Sync`
- `TransitionContext` is all value types, safe to construct
- No shared mutable state in enforcement checking
- Thread-safe by design

### Error Handling

The `IgnoreAndLog` strategy is complex because it needs to continue execution after violations. Consider:
- Log to stderr with `eprintln!`
- Consider adding optional logging callback in future
- Document that this strategy should only be used for non-critical checks

## Migration and Compatibility

### Breaking Changes

None. This is fixing non-functional existing code.

### API Compatibility

All existing public APIs remain unchanged:
- `StateMachine::new()` signature unchanged
- `step()` signature may change (see Implementation Notes)
- `apply_result()` signature unchanged
- `EnforcementBuilder` API unchanged

### Behavior Changes

**Before**: Enforcement rules silently ignored
**After**: Enforcement rules actually checked and enforced

This is a behavior change but fixes broken functionality. Users expecting enforcement to work will now get correct behavior.

### Migration Path

1. Review existing code using `EnforcementBuilder`
2. Test that enforcement behaves as expected
3. Adjust max_attempts/timeout values if needed
4. No code changes required

### Upgrade Notes

Add to CHANGELOG.md:

```markdown
## [0.2.0] - 2025-12-XX

### Fixed
- **CRITICAL**: Enforcement rules are now actually enforced during state machine execution
  - Previously, enforcement rules were silently ignored
  - If you were relying on enforcement, please test your machines
  - Transitions may now be blocked that previously succeeded
  - Adjust max_attempts and timeout values if needed

### Changed
- State machine now tracks transition start time for timeout checking
- Enforcement violations are properly accumulated and reported
```

## Success Metrics

- [ ] All 16 acceptance criteria met
- [ ] Test coverage for enforcement integration: >90%
- [ ] Example demonstrates real enforcement blocking transitions
- [ ] Documentation clearly explains enforcement integration
- [ ] Zero performance regression for machines without enforcement
- [ ] All existing tests pass
- [ ] Property tests verify violation accumulation invariants
