# Mindset Builder API Guide

A comprehensive guide to using Mindset's builder API for ergonomic state machine construction.

## Table of Contents

- [Getting Started](#getting-started)
- [Core Concepts](#core-concepts)
- [StateMachineBuilder](#statemachinebuilder)
- [TransitionBuilder](#transitionbuilder)
- [Common Patterns](#common-patterns)
- [Macro Reference](#macro-reference)
- [Advanced Usage](#advanced-usage)
- [Error Handling](#error-handling)

## Getting Started

The builder API provides a fluent interface for constructing state machines with minimal boilerplate while maintaining type safety. Instead of manually constructing state machine components, builders guide you through the required fields and validate at build time.

### Quick Example

```rust
use mindset::builder::{StateMachineBuilder, TransitionBuilder};
use mindset::state_enum;

// Define your states using the state_enum macro
state_enum! {
    enum OrderState {
        Draft,
        Submitted,
        Processing,
        Completed,
        Failed,
    }
    final: [Completed, Failed]
    error: [Failed]
}

// Build your state machine
let machine = StateMachineBuilder::new()
    .initial(OrderState::Draft)
    .add_transition(
        TransitionBuilder::new()
            .from(OrderState::Draft)
            .to(OrderState::Submitted)
            .succeeds()
            .build()
            .unwrap()
    )
    .add_transition(
        TransitionBuilder::new()
            .from(OrderState::Submitted)
            .to(OrderState::Processing)
            .succeeds()
            .build()
            .unwrap()
    )
    .build()
    .unwrap();
```

## Core Concepts

### Builder Pattern Benefits

1. **Compile-time validation**: Required fields are checked at build time
2. **Method chaining**: Fluent API for readable construction
3. **Type safety**: Generics ensure state and environment types match
4. **Clear errors**: Descriptive error messages for missing fields

### Builder Lifecycle

All builders follow a consistent lifecycle:

1. **Create** - Call `::new()` to create a builder
2. **Configure** - Chain methods to set required and optional fields
3. **Build** - Call `.build()` to validate and construct the final object

If required fields are missing, `.build()` returns a descriptive error.

## StateMachineBuilder

The `StateMachineBuilder` constructs complete state machines with initial states and transitions.

### Type Signature

```rust
pub struct StateMachineBuilder<S: State + 'static, Env: Clone + Send + Sync + 'static>
```

- `S`: Your state enum type (must implement `State` trait)
- `Env`: Environment type for effectful transitions (use `()` for pure machines)

### Required Fields

- **initial**: The initial state of the machine (required)
- **transitions**: At least one transition (required)

### Methods

#### `new() -> Self`

Creates a new builder instance.

```rust
let builder = StateMachineBuilder::<MyState, ()>::new();
```

#### `initial(state: S) -> Self`

Sets the initial state (required).

```rust
builder.initial(MyState::Start)
```

#### `add_transition(transition: Transition<S, Env>) -> Self`

Adds a pre-built transition to the machine.

```rust
builder.add_transition(my_transition)
```

#### `transition(builder: TransitionBuilder<S, Env>) -> Result<Self, BuildError>`

Adds a transition using a builder. Returns an error if the builder fails validation.

```rust
builder.transition(
    TransitionBuilder::new()
        .from(State::A)
        .to(State::B)
        .succeeds()
)?
```

#### `transitions(transitions: Vec<Transition<S, Env>>) -> Self`

Adds multiple transitions at once.

```rust
builder.transitions(vec![transition1, transition2])
```

#### `build() -> Result<StateMachine<S, Env>, BuildError>`

Validates and builds the state machine. Returns errors if:
- Initial state is not set
- No transitions are defined

### Complete Example

```rust
use mindset::builder::{StateMachineBuilder, TransitionBuilder};

state_enum! {
    enum LightState {
        Red,
        Yellow,
        Green,
    }
}

let machine = StateMachineBuilder::new()
    .initial(LightState::Red)
    .add_transition(
        TransitionBuilder::new()
            .from(LightState::Red)
            .to(LightState::Green)
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(LightState::Green)
            .to(LightState::Yellow)
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(LightState::Yellow)
            .to(LightState::Red)
            .succeeds()
            .build()?
    )
    .build()?;
```

## TransitionBuilder

The `TransitionBuilder` constructs individual state transitions with guards, actions, and enforcement rules.

### Type Signature

```rust
pub struct TransitionBuilder<S: State, Env>
```

### Required Fields

- **from**: Source state (required)
- **to**: Target state (required)
- **action**: Transition action/effect (required)

### Methods

#### `new() -> Self`

Creates a new transition builder.

```rust
let builder = TransitionBuilder::<MyState, ()>::new();
```

#### `from(state: S) -> Self`

Sets the source state (required).

```rust
builder.from(MyState::Draft)
```

#### `to(state: S) -> Self`

Sets the target state (required).

```rust
builder.to(MyState::Submitted)
```

#### `guard(guard: Guard<S>) -> Self`

Adds a guard object (optional).

```rust
builder.guard(my_guard)
```

#### `when<F>(predicate: F) -> Self`

Adds a guard using a closure (optional). More ergonomic than `guard()`.

```rust
builder.when(|state| state.is_ready())
```

#### `action<E>(effect: E) -> Self`

Sets a custom action effect (required, or use `succeeds()`).

```rust
builder.action(|| pure(TransitionResult::Success(MyState::Done)).boxed())
```

#### `succeeds() -> Self`

Sets a simple success action that transitions to the target state. Requires `.to()` to be called first.

```rust
builder
    .from(State::A)
    .to(State::B)
    .succeeds()  // Automatically creates success action to State::B
```

#### `enforce(rules: EnforcementRules<S>) -> Self`

Adds enforcement rules (optional).

```rust
use mindset::enforcement::EnforcementBuilder;
use std::time::Duration;

builder.enforce(
    EnforcementBuilder::new()
        .max_attempts(3)
        .timeout(Duration::from_secs(30))
        .build()
)
```

#### `build() -> Result<Transition<S, Env>, BuildError>`

Validates and builds the transition. Returns errors if:
- Source state is not set
- Target state is not set
- Action is not set

### Complete Example

```rust
use mindset::builder::TransitionBuilder;
use mindset::enforcement::EnforcementBuilder;
use std::time::Duration;

let transition = TransitionBuilder::new()
    .from(OrderState::Draft)
    .to(OrderState::Submitted)
    .when(|order| order.is_valid())
    .enforce(
        EnforcementBuilder::new()
            .max_attempts(3)
            .timeout(Duration::from_secs(10))
            .build()
    )
    .succeeds()
    .build()?;
```

## Common Patterns

### Pattern 1: Simple Linear Workflow

Build a straightforward sequence of states.

```rust
state_enum! {
    enum TaskState {
        Todo,
        InProgress,
        Review,
        Done,
    }
    final: [Done]
}

let machine = StateMachineBuilder::new()
    .initial(TaskState::Todo)
    .add_transition(
        TransitionBuilder::new()
            .from(TaskState::Todo)
            .to(TaskState::InProgress)
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(TaskState::InProgress)
            .to(TaskState::Review)
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(TaskState::Review)
            .to(TaskState::Done)
            .succeeds()
            .build()?
    )
    .build()?;
```

### Pattern 2: Branching with Guards

Add conditional transitions based on state properties.

```rust
state_enum! {
    enum PaymentState {
        Pending,
        Processing,
        Completed,
        Failed,
    }
    final: [Completed, Failed]
    error: [Failed]
}

struct Payment {
    amount: f64,
    state: PaymentState,
}

let machine = StateMachineBuilder::new()
    .initial(PaymentState::Pending)
    .add_transition(
        TransitionBuilder::new()
            .from(PaymentState::Pending)
            .to(PaymentState::Processing)
            .when(|_| true)  // Always allow this transition
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(PaymentState::Processing)
            .to(PaymentState::Completed)
            .when(|state| !state.is_error())  // Only if not in error state
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(PaymentState::Processing)
            .to(PaymentState::Failed)
            .when(|state| state.is_error())  // Only on error
            .succeeds()
            .build()?
    )
    .build()?;
```

### Pattern 3: Enforced Transitions

Add retry limits and timeouts to prevent infinite loops.

```rust
use mindset::enforcement::EnforcementBuilder;
use std::time::Duration;

state_enum! {
    enum ApiState {
        Idle,
        Requesting,
        Success,
        Error,
    }
    final: [Success, Error]
    error: [Error]
}

let machine = StateMachineBuilder::new()
    .initial(ApiState::Idle)
    .add_transition(
        TransitionBuilder::new()
            .from(ApiState::Idle)
            .to(ApiState::Requesting)
            .enforce(
                EnforcementBuilder::new()
                    .max_attempts(3)
                    .timeout(Duration::from_secs(30))
                    .build()
            )
            .succeeds()
            .build()?
    )
    .build()?;
```

### Pattern 4: Bulk Transition Creation

Create multiple similar transitions efficiently.

```rust
use mindset::builder::simple_transition;

state_enum! {
    enum ProcessState {
        Step1,
        Step2,
        Step3,
        Step4,
        Done,
    }
    final: [Done]
}

let transitions = vec![
    simple_transition(ProcessState::Step1, ProcessState::Step2),
    simple_transition(ProcessState::Step2, ProcessState::Step3),
    simple_transition(ProcessState::Step3, ProcessState::Step4),
    simple_transition(ProcessState::Step4, ProcessState::Done),
];

let machine = StateMachineBuilder::new()
    .initial(ProcessState::Step1)
    .transitions(transitions)
    .build()?;
```

### Pattern 5: Error Recovery Paths

Create multiple paths including error recovery.

```rust
state_enum! {
    enum DeployState {
        Ready,
        Deploying,
        Verifying,
        Complete,
        Failed,
        RollingBack,
        RolledBack,
    }
    final: [Complete, RolledBack]
    error: [Failed]
}

let machine = StateMachineBuilder::new()
    .initial(DeployState::Ready)
    // Happy path
    .add_transition(
        TransitionBuilder::new()
            .from(DeployState::Ready)
            .to(DeployState::Deploying)
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(DeployState::Deploying)
            .to(DeployState::Verifying)
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(DeployState::Verifying)
            .to(DeployState::Complete)
            .succeeds()
            .build()?
    )
    // Error path
    .add_transition(
        TransitionBuilder::new()
            .from(DeployState::Deploying)
            .to(DeployState::Failed)
            .when(|state| state.is_error())
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(DeployState::Failed)
            .to(DeployState::RollingBack)
            .succeeds()
            .build()?
    )
    .add_transition(
        TransitionBuilder::new()
            .from(DeployState::RollingBack)
            .to(DeployState::RolledBack)
            .succeeds()
            .build()?
    )
    .build()?;
```

## Macro Reference

### `state_enum!`

Generates a State trait implementation for simple enums, reducing boilerplate.

#### Syntax

```rust
state_enum! {
    $(#[$meta:meta])*              // Optional attributes (e.g., #[derive(...)])
    $vis:vis enum $name:ident {    // Visibility and enum name
        $(
            $(#[$variant_meta:meta])*  // Optional variant attributes
            $variant:ident
        ),*
    }

    final: [$($final:ident),*]     // Optional: List of final states
    error: [$($error:ident),*]     // Optional: List of error states
}
```

#### Features

1. **Automatic State Implementation**: Generates `State` trait implementation
2. **Serialization Support**: Automatically derives `Serialize` and `Deserialize`
3. **Debug Support**: Derives `Clone`, `PartialEq`, and `Debug`
4. **Final States**: Mark terminal states with `final: [...]`
5. **Error States**: Mark error states with `error: [...]`

#### Generated Methods

The macro generates implementations for:

- `fn name(&self) -> &str`: Returns the variant name as a string
- `fn is_final(&self) -> bool`: Returns true if state is in the `final` list
- `fn is_error(&self) -> bool`: Returns true if state is in the `error` list

#### Examples

##### Basic Usage

```rust
use mindset::state_enum;

state_enum! {
    enum SimpleState {
        Start,
        End,
    }
}
```

##### With Final States

```rust
state_enum! {
    enum WorkflowState {
        Initial,
        Processing,
        Done,
        Failed,
    }
    final: [Done, Failed]
}
```

##### With Error States

```rust
state_enum! {
    enum TaskState {
        Ready,
        Running,
        Success,
        Error,
        Timeout,
    }
    final: [Success, Error, Timeout]
    error: [Error, Timeout]
}
```

##### Public Enum

```rust
state_enum! {
    pub enum PublicState {
        Open,
        Closed,
    }
    final: [Closed]
}
```

##### With Documentation

```rust
state_enum! {
    /// Represents the state of a database connection
    pub enum ConnectionState {
        /// Not yet connected
        Disconnected,
        /// Currently establishing connection
        Connecting,
        /// Successfully connected
        Connected,
        /// Connection failed
        Failed,
    }
    final: [Connected, Failed]
    error: [Failed]
}
```

## Advanced Usage

### Custom Actions with Effects

For complex transitions that need to perform I/O or side effects:

```rust
use stillwater::prelude::*;
use mindset::effects::{TransitionResult, TransitionError};

trait Logger {
    fn log(&mut self, msg: &str);
}

let transition = TransitionBuilder::new()
    .from(State::A)
    .to(State::B)
    .action(|| {
        // Create an effect that logs and transitions
        effect(|env: &mut dyn Logger| {
            env.log("Transitioning from A to B");
            TransitionResult::Success(State::B)
        }).boxed()
    })
    .build()?;
```

### Combining Guards and Enforcement

Use both guards (pre-conditions) and enforcement (runtime limits):

```rust
let transition = TransitionBuilder::new()
    .from(State::Idle)
    .to(State::Processing)
    .when(|state| state.is_ready())  // Pre-condition check
    .enforce(
        EnforcementBuilder::new()
            .max_attempts(3)
            .timeout(Duration::from_secs(30))
            .require_pred(
                |ctx| !ctx.from.is_locked(),
                "State is locked".to_string()
            )
            .build()
    )
    .succeeds()
    .build()?;
```

### Helper Functions

Mindset provides helper functions for common transition patterns:

#### `simple_transition`

Creates an unconditional transition with no guard.

```rust
use mindset::builder::simple_transition;

let transition = simple_transition::<MyState, ()>(
    MyState::Start,
    MyState::End
);
```

#### `guarded_transition`

Creates a transition with a guard predicate.

```rust
use mindset::builder::guarded_transition;

let transition = guarded_transition::<MyState, (), _>(
    MyState::Start,
    MyState::Middle,
    |state| !state.is_final()
);
```

## Error Handling

### BuildError Types

All builders return `Result<T, BuildError>` from their `.build()` methods.

#### StateMachineBuilder Errors

- `BuildError::MissingInitialState`: Initial state not set
- `BuildError::NoTransitions`: No transitions added

```rust
let result = StateMachineBuilder::<MyState, ()>::new().build();

match result {
    Ok(machine) => { /* use machine */ },
    Err(BuildError::MissingInitialState) => {
        eprintln!("Must call .initial(state) before .build()");
    },
    Err(BuildError::NoTransitions) => {
        eprintln!("Must add at least one transition");
    },
    _ => {}
}
```

#### TransitionBuilder Errors

- `BuildError::MissingFromState`: Source state not set
- `BuildError::MissingToState`: Target state not set
- `BuildError::MissingAction`: Action not set

```rust
let result = TransitionBuilder::<MyState, ()>::new()
    .from(MyState::A)
    .build();

match result {
    Ok(transition) => { /* use transition */ },
    Err(BuildError::MissingToState) => {
        eprintln!("Must call .to(state) before .build()");
    },
    Err(BuildError::MissingAction) => {
        eprintln!("Must call .action() or .succeeds() before .build()");
    },
    _ => {}
}
```

### Error Recovery

Use the `?` operator for clean error propagation:

```rust
fn build_machine() -> Result<StateMachine<MyState, ()>, BuildError> {
    Ok(StateMachineBuilder::new()
        .initial(MyState::Start)
        .transition(
            TransitionBuilder::new()
                .from(MyState::Start)
                .to(MyState::End)
                .succeeds()
        )?  // Propagates TransitionBuilder errors
        .build()?)  // Propagates StateMachineBuilder errors
}
```

## Best Practices

1. **Use the `state_enum!` macro**: Reduces boilerplate and ensures consistency
2. **Prefer `.succeeds()` over custom actions**: Use the simpler API unless you need effects
3. **Chain builders with `?`**: Use the `?` operator for clean error handling
4. **Mark final states**: Always specify final states for proper termination
5. **Mark error states**: Distinguish error states from successful terminal states
6. **Use guards for pre-conditions**: Validate state before transitions
7. **Use enforcement for runtime limits**: Prevent infinite loops and timeouts
8. **Collect transitions in vectors**: For bulk operations and cleaner code
9. **Document complex guards**: Add comments explaining guard logic
10. **Test each transition**: Verify guards and actions work as expected

## Next Steps

- Review the [Effects Guide](effects-guide.md) for effectful transitions
- See the [Enforcement Guide](enforcement.md) for validation and policies
- Check the [API Documentation](https://docs.rs/mindset) for complete reference
- Explore the [Checkpointing Guide](checkpointing.md) for long-running workflows
