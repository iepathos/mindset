---
number: 002
title: Effect-Based Transitions
category: foundation
priority: critical
status: draft
dependencies: [001]
created: 2025-12-01
---

# Specification 002: Effect-Based Transitions

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: Specification 001 (Core State Machine Foundation)

## Context

Building on the pure state transition logic from Specification 001, we now add the "water" that flows through the system - effectful transitions that perform I/O, call external services, and execute side effects. This follows Stillwater 0.11.0's zero-cost effect system where effects are concrete types by default, with explicit boxing only when needed.

The transition system must:
1. Use Stillwater's **0.11.0 API** (not 0.10.x)
2. Return `impl Effect` for zero-cost composition
3. Box effects only when storing in collections
4. Separate pure guard logic from effectful actions

## Objective

Implement effectful state transitions using Stillwater 0.11.0's Effect system, providing a clean separation between pure guard predicates and effectful transition actions while maintaining zero-cost abstractions.

## Requirements

### Functional Requirements

- **Transition Type**: Model transitions with from/to states, guards, and effectful actions
- **Effect Integration**: Use Stillwater 0.11.0 constructors (`pure()`, `fail()`, `from_fn()`)
- **Transition Results**: Support Success, Retry, and Abort outcomes
- **State Machine**: Execute transitions and update state
- **Step Execution**: Single-step transition execution with effect composition
- **Environment Generic**: Support generic environment types for dependency injection

### Non-Functional Requirements

- **Zero-Cost by Default**: Use `impl Effect` for return types
- **Explicit Boxing**: Only box when storing in collections
- **Pure Guards**: Guards remain pure predicates from Spec 001
- **Type Safety**: Leverage Rust's type system to prevent invalid transitions
- **Async Support**: Integrate with Stillwater's async effect execution

## Acceptance Criteria

- [ ] `Transition<S, Env>` struct with guard and boxed action
- [ ] `TransitionResult<S>` enum with Success/Retry/Abort variants
- [ ] `StateMachine<S, Env>` with current state and transitions
- [ ] `step()` method returns `impl Effect` for zero-cost execution
- [ ] Transitions stored with `BoxedEffect` for collection storage
- [ ] Guards evaluated before action execution (pure check)
- [ ] History updated after successful transitions
- [ ] Uses Stillwater 0.11.0 API (`pure()`, `fail()`, `from_fn()`)
- [ ] All tests pass with mock environments
- [ ] Documentation shows zero-cost vs boxed patterns

## Technical Details

### Implementation Approach

Follow Stillwater's "futures crate pattern":
- Return `impl Effect` from functions
- Store `BoxedEffect` in structs
- Use free-standing constructors (`pure()`, not `Effect::pure()`)

### Transition Result Type

```rust
use serde::{Deserialize, Serialize};

/// Result of executing a transition action.
/// Returned from effectful transition logic.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransitionResult<S: State> {
    /// Transition succeeded, move to new state
    Success(S),

    /// Transition should be retried with feedback
    Retry {
        feedback: String,
        current_state: S,
    },

    /// Transition failed permanently
    Abort {
        reason: String,
        error_state: S,
    },
}
```

### Transition Type

```rust
use stillwater::effect::{BoxedEffect, Effect};
use crate::core::{Guard, State};

/// A transition from one state to another with an effectful action.
/// Transitions are stored in collections, so actions are boxed.
pub struct Transition<S: State, Env> {
    pub from: S,
    pub to: S,
    pub guard: Option<Guard<S>>,
    pub action: BoxedEffect<TransitionResult<S>, TransitionError, Env>,
}

impl<S: State, Env> Transition<S, Env> {
    /// Check if this transition can execute from the current state (pure)
    pub fn can_execute(&self, current: &S) -> bool {
        // Check state match
        if *current != self.from {
            return false;
        }

        // Check guard if present (pure predicate)
        self.guard.as_ref().map_or(true, |g| g.check(current))
    }
}

/// Errors that can occur during transitions
#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("No transition available from state '{from}'")]
    NoTransition { from: String },

    #[error("Guard blocked transition from '{from}' to '{to}'")]
    GuardBlocked { from: String, to: String },

    #[error("Transition action failed: {0}")]
    ActionFailed(String),
}
```

### State Machine Type

```rust
use stillwater::effect::Effect;
use stillwater::prelude::*;
use crate::core::{State, StateHistory, StateTransition};
use chrono::Utc;

/// State machine that executes effectful transitions.
pub struct StateMachine<S: State, Env> {
    initial: S,
    current: S,
    transitions: Vec<Transition<S, Env>>,
    history: StateHistory<S>,
    attempt_count: usize,
}

impl<S: State, Env> StateMachine<S, Env> {
    /// Create a new state machine in the initial state
    pub fn new(initial: S) -> Self {
        Self {
            initial: initial.clone(),
            current: initial,
            transitions: Vec::new(),
            history: StateHistory::new(),
            attempt_count: 0,
        }
    }

    /// Add a transition to the machine
    pub fn add_transition(&mut self, transition: Transition<S, Env>) {
        self.transitions.push(transition);
    }

    /// Get current state (pure)
    pub fn current_state(&self) -> &S {
        &self.current
    }

    /// Check if machine is in a final state (pure)
    pub fn is_final(&self) -> bool {
        self.current.is_final()
    }

    /// Get state history (pure)
    pub fn history(&self) -> &StateHistory<S> {
        &self.history
    }

    /// Execute one step of the state machine.
    /// Returns impl Effect for zero-cost composition.
    pub fn step(&mut self) -> impl Effect<Output = StepResult<S>, Error = TransitionError, Env = Env> + '_ {
        // Find applicable transition (pure)
        let transition_opt = self.transitions.iter()
            .find(|t| t.can_execute(&self.current));

        let Some(transition) = transition_opt else {
            return fail(TransitionError::NoTransition {
                from: self.current.name().to_string(),
            }).boxed();
        };

        // Clone what we need for the effect
        let from_state = self.current.clone();
        let to_state = transition.to.clone();
        let action = transition.action.clone();

        // Execute action effect and update machine state
        action.and_then(move |result| {
            pure(result)
        }).map(move |result| {
            match result {
                TransitionResult::Success(new_state) => {
                    // Record transition
                    let transition_record = StateTransition {
                        from: from_state.clone(),
                        to: new_state.clone(),
                        timestamp: Utc::now(),
                        attempt: self.attempt_count,
                    };

                    self.history = self.history.record(transition_record);
                    self.current = new_state.clone();
                    self.attempt_count = 0;

                    StepResult::Transitioned(new_state)
                }
                TransitionResult::Retry { feedback, current_state } => {
                    self.attempt_count += 1;
                    StepResult::Retry {
                        feedback,
                        attempts: self.attempt_count,
                    }
                }
                TransitionResult::Abort { reason, error_state } => {
                    self.current = error_state.clone();
                    StepResult::Aborted {
                        reason,
                        error_state,
                    }
                }
            }
        }).boxed()
    }
}

/// Result of executing a single step
#[derive(Clone, Debug, PartialEq)]
pub enum StepResult<S: State> {
    /// Successfully transitioned to new state
    Transitioned(S),

    /// Transition should be retried
    Retry {
        feedback: String,
        attempts: usize,
    },

    /// Transition aborted permanently
    Aborted {
        reason: String,
        error_state: S,
    },
}
```

### Module Structure

```
mindset/
├── src/
│   ├── core/               # Pure logic (from Spec 001)
│   ├── effects/            # Effectful operations
│   │   ├── mod.rs
│   │   ├── transition.rs   # Transition and TransitionResult
│   │   └── machine.rs      # StateMachine
│   └── lib.rs
```

## Dependencies

- **Prerequisites**: Specification 001 (Core State Machine Foundation)
- **Affected Components**: None (new module)
- **External Dependencies**:
  - `stillwater = { version = "0.11", features = ["async"] }`
  - `thiserror = "1.0"`

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use stillwater::prelude::*;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum WorkflowState {
        Initial,
        Processing,
        Complete,
        Failed,
    }

    impl State for WorkflowState {
        fn name(&self) -> &str {
            match self {
                Self::Initial => "Initial",
                Self::Processing => "Processing",
                Self::Complete => "Complete",
                Self::Failed => "Failed",
            }
        }

        fn is_final(&self) -> bool {
            matches!(self, Self::Complete | Self::Failed)
        }
    }

    #[derive(Clone)]
    struct TestEnv {
        should_succeed: bool,
    }

    #[tokio::test]
    async fn simple_transition_succeeds() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: pure(TransitionResult::Success(WorkflowState::Processing)).boxed(),
        };

        machine.add_transition(transition);

        let env = TestEnv { should_succeed: true };
        let result = machine.step().run(&env).await;

        assert!(result.is_ok());
        assert_eq!(machine.current_state(), &WorkflowState::Processing);
        assert_eq!(machine.history().transitions().len(), 1);
    }

    #[tokio::test]
    async fn guard_blocks_transition() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let guard = Guard::new(|s: &WorkflowState| s.is_final());

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: Some(guard),
            action: pure(TransitionResult::Success(WorkflowState::Processing)).boxed(),
        };

        machine.add_transition(transition);

        let env = TestEnv { should_succeed: true };
        let result = machine.step().run(&env).await;

        // Should fail because Initial is not final
        assert!(result.is_err());
        assert_eq!(machine.current_state(), &WorkflowState::Initial);
    }

    #[tokio::test]
    async fn retry_increments_attempt_count() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: pure(TransitionResult::Retry {
                feedback: "Not ready yet".to_string(),
                current_state: WorkflowState::Initial,
            }).boxed(),
        };

        machine.add_transition(transition);

        let env = TestEnv { should_succeed: false };
        let result = machine.step().run(&env).await.unwrap();

        match result {
            StepResult::Retry { attempts, .. } => assert_eq!(attempts, 1),
            _ => panic!("Expected Retry result"),
        }

        // Second attempt
        let result2 = machine.step().run(&env).await.unwrap();
        match result2 {
            StepResult::Retry { attempts, .. } => assert_eq!(attempts, 2),
            _ => panic!("Expected Retry result"),
        }
    }

    #[tokio::test]
    async fn effectful_action_with_environment() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: from_fn(|env: &TestEnv| {
                if env.should_succeed {
                    Ok(TransitionResult::Success(WorkflowState::Processing))
                } else {
                    Ok(TransitionResult::Abort {
                        reason: "Environment not ready".to_string(),
                        error_state: WorkflowState::Failed,
                    })
                }
            }).boxed(),
        };

        machine.add_transition(transition);

        let env = TestEnv { should_succeed: true };
        let result = machine.step().run(&env).await.unwrap();

        assert!(matches!(result, StepResult::Transitioned(_)));
        assert_eq!(machine.current_state(), &WorkflowState::Processing);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn multi_step_workflow() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        // Initial -> Processing
        machine.add_transition(Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: pure(TransitionResult::Success(WorkflowState::Processing)).boxed(),
        });

        // Processing -> Complete
        machine.add_transition(Transition {
            from: WorkflowState::Processing,
            to: WorkflowState::Complete,
            guard: None,
            action: pure(TransitionResult::Success(WorkflowState::Complete)).boxed(),
        });

        let env = TestEnv { should_succeed: true };

        // First step
        machine.step().run(&env).await.unwrap();
        assert_eq!(machine.current_state(), &WorkflowState::Processing);

        // Second step
        machine.step().run(&env).await.unwrap();
        assert_eq!(machine.current_state(), &WorkflowState::Complete);
        assert!(machine.is_final());

        // Verify history
        let path = machine.history().get_path();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], &WorkflowState::Initial);
        assert_eq!(path[1], &WorkflowState::Processing);
        assert_eq!(path[2], &WorkflowState::Complete);
    }
}
```

## Documentation Requirements

### Code Documentation

- Document `impl Effect` vs `BoxedEffect` usage patterns
- Show examples of zero-cost effect composition
- Explain when to use `.boxed()`

### User Documentation

Create `docs/effects-guide.md`:
- Stillwater 0.11.0 effect patterns
- Zero-cost abstractions explained
- Environment and dependency injection
- Example effectful transitions

### Architecture Updates

Update README.md:
- Effect-based transition model
- Pure guards vs effectful actions
- Zero-cost by default philosophy

## Implementation Notes

### Stillwater 0.11.0 Migration

This uses the **latest** Stillwater API:

```rust
// ✅ Correct (0.11.0):
use stillwater::prelude::*;
pure(value)                    // Not Effect::pure()
fail(error)                    // Not Effect::fail()
from_fn(|env| Ok(value))       // Not Effect::from_fn()

// Return types:
fn zero_cost() -> impl Effect<Output=T, Error=E, Env=Env> { ... }
fn boxed() -> BoxedEffect<T, E, Env> { ... }
```

### Pure vs Effect Boundary

**Pure (Spec 001)**:
- Guard predicates
- State trait methods
- History operations

**Effects (Spec 002)**:
- Transition actions
- State machine step execution
- Any I/O or external calls

### Performance

- `impl Effect` has zero runtime cost
- `BoxedEffect` has one heap allocation per transition
- This is acceptable for state machine use case
- Transitions are not performance-critical hot path

## Migration and Compatibility

No migration needed (new library). Design ensures:
- Future parallel transitions (Effects compose)
- Future async/await integration (already supported)
- Future optimization (can replace Vec with HashMap)
