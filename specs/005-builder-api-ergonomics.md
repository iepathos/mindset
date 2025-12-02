---
number: 005
title: Builder API and Ergonomics
category: foundation
priority: medium
status: draft
dependencies: [001, 002, 003, 004]
created: 2025-12-01
---

# Specification 005: Builder API and Ergonomics

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Specification 001, 002, 003, 004

## Context

While the core state machine API is functional, it requires verbose setup:
```rust
let mut machine = StateMachine::new(State::Initial);

let transition = Transition {
    from: State::Initial,
    to: State::Processing,
    guard: Some(Guard::new(|s| !s.is_final())),
    action: pure(TransitionResult::Success(State::Processing)).boxed(),
    enforcement: Some(EnforcementRules::builder()...),
};

machine.add_transition(transition);
```

This creates friction for users. We need a fluent, ergonomic API that:
- Reduces boilerplate
- Provides type inference
- Guides correct usage
- Makes common patterns easy
- Keeps advanced features accessible

## Objective

Create a fluent builder API for constructing state machines and transitions, with convenience macros for common patterns, comprehensive error messages, and excellent type inference.

## Requirements

### Functional Requirements

- **StateMachine Builder**: Fluent API for building machines
- **Transition Builder**: Fluent API for building transitions
- **State Enum Macro**: Generate State trait implementations
- **Type Inference**: Minimize type annotations needed
- **Error Messages**: Clear, actionable compile errors
- **Convenience Methods**: Common patterns as one-liners
- **Backwards Compatible**: Don't break existing code

### Non-Functional Requirements

- **Ergonomic**: Feel natural to Rust developers
- **Discoverable**: IDE autocomplete guides usage
- **Type Safe**: Invalid configurations caught at compile time
- **Documentation**: Every public method has examples
- **Zero Runtime Cost**: Builders optimize away

## Acceptance Criteria

- [ ] `StateMachineBuilder<S, Env>` with fluent methods
- [ ] `TransitionBuilder<S, Env>` with fluent methods
- [ ] `state_enum!` macro generates State implementations
- [ ] `simple_transition!` macro for common cases
- [ ] Builder validation at `build()` time
- [ ] Clear error messages for builder misuse
- [ ] Type inference works for common patterns
- [ ] Documentation examples compile and run
- [ ] All builder tests pass
- [ ] Zero overhead vs manual construction

## Technical Details

### Implementation Approach

Use the typestate pattern for compile-time validation where beneficial, but prefer runtime validation for simplicity.

### StateMachine Builder

```rust
use std::marker::PhantomData;
use crate::core::State;
use crate::effects::{StateMachine, Transition};

/// Builder for constructing state machines
pub struct StateMachineBuilder<S: State, Env> {
    initial: Option<S>,
    transitions: Vec<Transition<S, Env>>,
    _phantom: PhantomData<Env>,
}

impl<S: State, Env> StateMachineBuilder<S, Env> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            initial: None,
            transitions: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Set the initial state (required)
    pub fn initial(mut self, state: S) -> Self {
        self.initial = Some(state);
        self
    }

    /// Add a transition using a builder
    pub fn transition(mut self, builder: TransitionBuilder<S, Env>) -> Result<Self, BuildError> {
        let transition = builder.build()?;
        self.transitions.push(transition);
        Ok(self)
    }

    /// Add a pre-built transition
    pub fn add_transition(mut self, transition: Transition<S, Env>) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Add multiple transitions at once
    pub fn transitions(mut self, transitions: Vec<Transition<S, Env>>) -> Self {
        self.transitions.extend(transitions);
        self
    }

    /// Build the state machine
    pub fn build(self) -> Result<StateMachine<S, Env>, BuildError> {
        let initial = self.initial.ok_or(BuildError::MissingInitialState)?;

        if self.transitions.is_empty() {
            return Err(BuildError::NoTransitions);
        }

        let mut machine = StateMachine::new(initial);
        for transition in self.transitions {
            machine.add_transition(transition);
        }

        Ok(machine)
    }
}

impl<S: State, Env> Default for StateMachineBuilder<S, Env> {
    fn default() -> Self {
        Self::new()
    }
}
```

### Transition Builder

```rust
use stillwater::effect::{BoxedEffect, Effect};
use stillwater::prelude::*;
use crate::core::{Guard, State};
use crate::effects::{Transition, TransitionResult, TransitionError};
use crate::enforcement::EnforcementRules;

/// Builder for constructing transitions
pub struct TransitionBuilder<S: State, Env> {
    from: Option<S>,
    to: Option<S>,
    guard: Option<Guard<S>>,
    action: Option<BoxedEffect<TransitionResult<S>, TransitionError, Env>>,
    enforcement: Option<EnforcementRules<S>>,
}

impl<S: State, Env> TransitionBuilder<S, Env> {
    /// Create a new transition builder
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            guard: None,
            action: None,
            enforcement: None,
        }
    }

    /// Set the source state (required)
    pub fn from(mut self, state: S) -> Self {
        self.from = Some(state);
        self
    }

    /// Set the target state (required)
    pub fn to(mut self, state: S) -> Self {
        self.to = Some(state);
        self
    }

    /// Add a guard predicate (optional)
    pub fn guard(mut self, guard: Guard<S>) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Add a guard using a closure
    pub fn when<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&S) -> bool + Send + Sync + 'static,
    {
        self.guard = Some(Guard::new(predicate));
        self
    }

    /// Set the action effect (required)
    pub fn action<E>(mut self, effect: E) -> Self
    where
        E: Effect<Output = TransitionResult<S>, Error = TransitionError, Env = Env> + 'static,
    {
        self.action = Some(effect.boxed());
        self
    }

    /// Set a simple success action
    pub fn succeeds(self) -> Self {
        let to = self.to.clone().expect("to() must be called before succeeds()");
        self.action(pure(TransitionResult::Success(to)))
    }

    /// Add enforcement rules
    pub fn enforce(mut self, rules: EnforcementRules<S>) -> Self {
        self.enforcement = Some(rules);
        self
    }

    /// Build the transition
    pub fn build(self) -> Result<Transition<S, Env>, BuildError> {
        let from = self.from.ok_or(BuildError::MissingFromState)?;
        let to = self.to.ok_or(BuildError::MissingToState)?;
        let action = self.action.ok_or(BuildError::MissingAction)?;

        Ok(Transition {
            from,
            to,
            guard: self.guard,
            action,
            enforcement: self.enforcement,
        })
    }
}

impl<S: State, Env> Default for TransitionBuilder<S, Env> {
    fn default() -> Self {
        Self::new()
    }
}
```

### Build Errors

```rust
use thiserror::Error;

/// Errors that can occur when building state machines
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Initial state not specified. Call .initial(state) before .build()")]
    MissingInitialState,

    #[error("No transitions defined. Add at least one transition")]
    NoTransitions,

    #[error("Transition source state not specified. Call .from(state)")]
    MissingFromState,

    #[error("Transition target state not specified. Call .to(state)")]
    MissingToState,

    #[error("Transition action not specified. Call .action(effect) or .succeeds()")]
    MissingAction,
}
```

### State Enum Macro

```rust
/// Generate State trait implementation for simple enums
#[macro_export]
macro_rules! state_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),* $(,)?
        }

        $(final: [$($final:ident),* $(,)?])?
        $(error: [$($error:ident),* $(,)?])?
    ) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),*
        }

        impl $crate::core::State for $name {
            fn name(&self) -> &str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }

            fn is_final(&self) -> bool {
                match self {
                    $($(Self::$final => true,)*)?
                    _ => false,
                }
            }

            fn is_error(&self) -> bool {
                match self {
                    $($(Self::$error => true,)*)?
                    _ => false,
                }
            }
        }
    };
}
```

### Convenience Functions

```rust
/// Create a simple unconditional transition that succeeds
pub fn simple_transition<S, Env>(
    from: S,
    to: S,
) -> Transition<S, Env>
where
    S: State,
{
    TransitionBuilder::new()
        .from(from)
        .to(to.clone())
        .action(pure(TransitionResult::Success(to)))
        .build()
        .expect("Simple transition should always build")
}

/// Create a transition with a guard
pub fn guarded_transition<S, Env, F>(
    from: S,
    to: S,
    guard: F,
) -> Transition<S, Env>
where
    S: State,
    F: Fn(&S) -> bool + Send + Sync + 'static,
{
    TransitionBuilder::new()
        .from(from)
        .to(to.clone())
        .when(guard)
        .action(pure(TransitionResult::Success(to)))
        .build()
        .expect("Guarded transition should always build")
}
```

### Module Structure

```
mindset/
├── src/
│   ├── builder/
│   │   ├── mod.rs
│   │   ├── machine.rs       # StateMachineBuilder
│   │   ├── transition.rs    # TransitionBuilder
│   │   ├── error.rs         # BuildError
│   │   └── macros.rs        # state_enum! macro
│   └── lib.rs               # Re-export builders in prelude
```

## Dependencies

- **Prerequisites**: Specification 001, 002, 003, 004
- **Affected Components**: None (pure addition)
- **External Dependencies**: None (uses existing deps)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    state_enum! {
        enum TestState {
            Initial,
            Processing,
            Complete,
            Failed,
        }
        final: [Complete, Failed]
        error: [Failed]
    }

    #[test]
    fn state_enum_macro_generates_trait() {
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
    fn builder_validates_required_fields() {
        let result = StateMachineBuilder::<TestState, ()>::new()
            .build();

        assert!(matches!(result, Err(BuildError::MissingInitialState)));
    }

    #[test]
    fn fluent_api_builds_machine() {
        let machine = StateMachineBuilder::new()
            .initial(TestState::Initial)
            .add_transition(simple_transition(
                TestState::Initial,
                TestState::Processing,
            ))
            .add_transition(simple_transition(
                TestState::Processing,
                TestState::Complete,
            ))
            .build();

        assert!(machine.is_ok());
        let machine = machine.unwrap();
        assert_eq!(machine.current_state(), &TestState::Initial);
    }

    #[test]
    fn transition_builder_validates_fields() {
        let result = TransitionBuilder::<TestState, ()>::new()
            .from(TestState::Initial)
            // Missing .to()
            .succeeds()
            .build();

        // Should panic because to() not called
    }

    #[test]
    fn transition_builder_with_guard() {
        let transition = TransitionBuilder::new()
            .from(TestState::Initial)
            .to(TestState::Processing)
            .when(|s: &TestState| !s.is_final())
            .succeeds()
            .build()
            .unwrap();

        assert!(transition.can_execute(&TestState::Initial));
        assert!(!transition.can_execute(&TestState::Complete));
    }
}
```

### Documentation Tests

```rust
/// # Example
///
/// ```
/// use mindset::prelude::*;
///
/// state_enum! {
///     enum WorkflowState {
///         Start,
///         Processing,
///         Done,
///     }
///     final: [Done]
/// }
///
/// let machine = StateMachineBuilder::new()
///     .initial(WorkflowState::Start)
///     .add_transition(simple_transition(
///         WorkflowState::Start,
///         WorkflowState::Processing,
///     ))
///     .add_transition(simple_transition(
///         WorkflowState::Processing,
///         WorkflowState::Done,
///     ))
///     .build()
///     .unwrap();
/// ```
pub struct StateMachineBuilder;
```

## Documentation Requirements

### Code Documentation

- Every builder method has example
- Error types explain how to fix
- Macro usage examples

### User Documentation

Create `docs/builder-guide.md`:
- Getting started with builders
- Common patterns and recipes
- When to use builders vs direct construction
- Macro reference

Update `docs/quick-start.md`:
- Use builder API in all examples
- Show progression from simple to advanced

### Architecture Updates

Update README.md:
- Quick start uses builder API
- Examples show ergonomic patterns
- Link to builder guide

## Implementation Notes

### Design Philosophy

**Progressive Disclosure**:
- Simple cases should be trivial
- Complex cases should be possible
- Don't force users through complexity

**Type Safety**:
- Prefer runtime errors with clear messages
- Use compile-time errors only when beneficial
- Don't sacrifice ergonomics for type safety

**Discoverability**:
- Method names follow Rust conventions
- IDE autocomplete guides usage
- Error messages tell you what to do

### Macro Hygiene

The `state_enum!` macro should:
- Be hygienic (no name conflicts)
- Work with visibility modifiers
- Support doc comments
- Generate clean code

### Zero Cost

Builders should optimize away:
```rust
// This:
let machine = StateMachineBuilder::new()
    .initial(State::A)
    .add_transition(t1)
    .build()?;

// Should compile to same code as:
let mut machine = StateMachine::new(State::A);
machine.add_transition(t1);
```

## Migration and Compatibility

This is purely additive - existing code continues to work. Users can adopt builders incrementally:

```rust
// Before:
let mut machine = StateMachine::new(State::Initial);
machine.add_transition(transition);

// After (optional):
let machine = StateMachineBuilder::new()
    .initial(State::Initial)
    .add_transition(transition)
    .build()?;
```

Recommend builders for new code, but don't force migration.
