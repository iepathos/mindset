---
number: 001
title: Core State Machine Foundation
category: foundation
priority: critical
status: draft
dependencies: []
created: 2025-12-01
---

# Specification 001: Core State Machine Foundation

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: None

## Context

Mindset is a pure functional state machine library built on Stillwater's Effect system. The foundation must establish core types and pure state transition logic that separates business rules from side effects. This follows Stillwater's "pure core, imperative shell" philosophy where state transitions are pure functions and side effects are isolated in Effect monads.

The state machine will be used by Platypus for AI workflow orchestration but is designed as a general-purpose library. The core foundation must be simple, testable, and composable.

## Objective

Implement the foundational state machine types with pure state transition logic, type-safe state representation, and comprehensive history tracking. This forms the "still water" core that remains calm and predictable.

## Requirements

### Functional Requirements

- **State Trait**: Define a trait for states with pure methods (name, is_final, is_error)
- **State Transitions**: Model transitions with from/to states and pure guard predicates
- **History Tracking**: Record all state transitions with timestamps and metadata
- **Guard Predicates**: Pure boolean functions that determine transition eligibility
- **Type Safety**: Prevent invalid state transitions at compile time where possible

### Non-Functional Requirements

- **Pure Logic**: All state transition logic must be pure (no side effects)
- **Zero-Cost Abstractions**: Use Rust's zero-cost guarantees via generics
- **Serializable**: All core types must support serde serialization
- **Thread-Safe**: Core types should be Send + Sync where applicable
- **Performance**: State transitions should be < 1μs for simple cases

## Acceptance Criteria

- [ ] `State` trait defined with `name()`, `is_final()`, `is_error()` methods
- [ ] States implement Clone, PartialEq, Debug, Serialize, DeserializeOwned
- [ ] `Guard<S>` type for pure pre-condition predicates
- [ ] `StateTransition<S>` records from/to states with timestamp
- [ ] `StateHistory<S>` maintains ordered transition log
- [ ] History provides `get_path()` returning state sequence
- [ ] History provides `duration()` calculating total elapsed time
- [ ] All types are serializable with serde
- [ ] 100% test coverage for pure state transition logic
- [ ] Documentation examples compile and demonstrate usage

## Technical Details

### Implementation Approach

Follow Stillwater's philosophy of "composition over complexity" - build from small, focused, pure functions.

### Core Types

```rust
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait for state machine states.
/// All methods are pure - no side effects.
pub trait State: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de> {
    /// Get the state's name for display/logging
    fn name(&self) -> &str;

    /// Check if this is a final (terminal) state
    fn is_final(&self) -> bool {
        false
    }

    /// Check if this is an error state
    fn is_error(&self) -> bool {
        false
    }
}
```

### Guard Predicates

```rust
use std::marker::PhantomData;

/// Pure predicate that determines if a transition can execute.
/// Guards are evaluated before attempting transition.
pub struct Guard<S: State> {
    predicate: Box<dyn Fn(&S) -> bool + Send + Sync>,
    _phantom: PhantomData<S>,
}

impl<S: State> Guard<S> {
    /// Create a guard from a pure predicate function
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn(&S) -> bool + Send + Sync + 'static,
    {
        Guard {
            predicate: Box::new(predicate),
            _phantom: PhantomData,
        }
    }

    /// Check if the guard allows transition from this state (pure)
    pub fn check(&self, state: &S) -> bool {
        (self.predicate)(state)
    }
}
```

### State History

```rust
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Record of a single state transition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateTransition<S: State> {
    pub from: S,
    pub to: S,
    pub timestamp: DateTime<Utc>,
    pub attempt: usize,
}

/// Ordered history of state transitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateHistory<S: State> {
    transitions: Vec<StateTransition<S>>,
}

impl<S: State> StateHistory<S> {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }

    /// Record a transition (pure - returns new history)
    pub fn record(&self, transition: StateTransition<S>) -> Self {
        let mut transitions = self.transitions.clone();
        transitions.push(transition);
        Self { transitions }
    }

    /// Get the path of states traversed
    pub fn get_path(&self) -> Vec<&S> {
        let mut path = Vec::new();
        if let Some(first) = self.transitions.first() {
            path.push(&first.from);
        }
        for transition in &self.transitions {
            path.push(&transition.to);
        }
        path
    }

    /// Calculate total duration from first to last transition
    pub fn duration(&self) -> Option<Duration> {
        if let (Some(first), Some(last)) = (self.transitions.first(), self.transitions.last()) {
            let duration = last.timestamp.signed_duration_since(first.timestamp);
            duration.to_std().ok()
        } else {
            None
        }
    }

    /// Get all transitions
    pub fn transitions(&self) -> &[StateTransition<S>] {
        &self.transitions
    }
}
```

### Module Structure

```
mindset/
├── src/
│   ├── core/               # Pure business logic
│   │   ├── mod.rs          # Re-exports
│   │   ├── state.rs        # State trait
│   │   ├── guard.rs        # Guard predicates
│   │   └── history.rs      # History tracking
│   └── lib.rs              # Public API
```

## Dependencies

- **Prerequisites**: None (foundational specification)
- **Affected Components**: None (new library)
- **External Dependencies**:
  - `serde = { version = "1.0", features = ["derive"] }`
  - `chrono = { version = "0.4", features = ["serde"] }`

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum TestState {
        Initial,
        Processing,
        Complete,
        Failed,
    }

    impl State for TestState {
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

        fn is_error(&self) -> bool {
            matches!(self, Self::Failed)
        }
    }

    #[test]
    fn state_trait_methods_work() {
        let state = TestState::Initial;
        assert_eq!(state.name(), "Initial");
        assert!(!state.is_final());
        assert!(!state.is_error());

        let complete = TestState::Complete;
        assert!(complete.is_final());
        assert!(!complete.is_error());

        let failed = TestState::Failed;
        assert!(failed.is_final());
        assert!(failed.is_error());
    }

    #[test]
    fn guard_allows_transition() {
        let state = TestState::Initial;
        let guard = Guard::new(|s: &TestState| !s.is_final());

        assert!(guard.check(&state));
        assert!(!guard.check(&TestState::Complete));
    }

    #[test]
    fn history_tracks_transitions() {
        let mut history = StateHistory::new();

        let transition1 = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: Utc::now(),
            attempt: 1,
        };

        history = history.record(transition1);

        let transition2 = StateTransition {
            from: TestState::Processing,
            to: TestState::Complete,
            timestamp: Utc::now(),
            attempt: 1,
        };

        history = history.record(transition2);

        let path = history.get_path();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], &TestState::Initial);
        assert_eq!(path[1], &TestState::Processing);
        assert_eq!(path[2], &TestState::Complete);
    }

    #[test]
    fn history_calculates_duration() {
        let history = StateHistory::new();
        let start = Utc::now();

        let transition = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: start,
            attempt: 1,
        };

        let history = history.record(transition);

        // Small delay
        std::thread::sleep(std::time::Duration::from_millis(10));

        let transition2 = StateTransition {
            from: TestState::Processing,
            to: TestState::Complete,
            timestamp: Utc::now(),
            attempt: 1,
        };

        let history = history.record(transition2);

        let duration = history.duration();
        assert!(duration.is_some());
        assert!(duration.unwrap() >= std::time::Duration::from_millis(10));
    }

    #[test]
    fn state_serializes_correctly() {
        let state = TestState::Initial;
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: TestState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}
```

### Property-Based Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arbitrary_state()(variant in 0..4u8) -> TestState {
            match variant {
                0 => TestState::Initial,
                1 => TestState::Processing,
                2 => TestState::Complete,
                _ => TestState::Failed,
            }
        }
    }

    proptest! {
        #[test]
        fn guard_is_deterministic(state in arbitrary_state()) {
            let guard = Guard::new(|s: &TestState| !s.is_final());
            let result1 = guard.check(&state);
            let result2 = guard.check(&state);
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn history_preserves_order(
            transitions in prop::collection::vec(arbitrary_state(), 1..10)
        ) {
            let mut history = StateHistory::new();
            let mut expected_path = vec![TestState::Initial];

            for (i, to_state) in transitions.iter().enumerate() {
                let from_state = if i == 0 {
                    TestState::Initial
                } else {
                    transitions[i - 1].clone()
                };

                let transition = StateTransition {
                    from: from_state.clone(),
                    to: to_state.clone(),
                    timestamp: Utc::now(),
                    attempt: 1,
                };

                history = history.record(transition);
                expected_path.push(to_state.clone());
            }

            let path = history.get_path();
            prop_assert_eq!(path.len(), expected_path.len());
        }
    }
}
```

## Documentation Requirements

### Code Documentation

- All public types have rustdoc comments
- Examples in doc comments compile and run
- Module-level documentation explains pure core philosophy

### User Documentation

Create `docs/core-concepts.md`:
- Explain State trait and implementation
- Show guard predicate patterns
- Demonstrate history tracking usage

### Architecture Updates

Document in README.md:
- Pure core design philosophy
- Zero-cost abstraction strategy
- Integration with Stillwater patterns

## Implementation Notes

### Pure Functions Only

This specification covers ONLY pure logic:
- State definitions
- Guard predicates
- History tracking
- No Effects, no I/O

Effects will be introduced in Specification 002.

### Immutability

Follow functional programming principles:
- History is immutable - `record()` returns new history
- Guards are pure predicates with no state
- State transitions are values, not mutations

### Performance Considerations

- Use `&str` for state names (zero-copy)
- Guards use `Box<dyn Fn>` for flexibility (acceptable cost)
- History uses `Vec` - linear scan is fine for typical state machine sizes
- Consider `SmallVec` optimization if profiling shows need

## Migration and Compatibility

This is a new library - no migration needed. However, the design ensures:
- Future async support (State is Send + Sync)
- Future optimization (History could use rope/tree structure)
- Future extensions (State trait could add metadata methods)
