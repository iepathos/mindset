# Mindset

A zero-cost, effect-based state machine library for Rust.

## Overview

Mindset provides a flexible and type-safe state machine implementation that separates pure guard logic from effectful actions. Built on Stillwater 0.11.0's effect system, it enables you to write state machines that are:

- **Zero-cost by default**: No runtime overhead when effects aren't needed
- **Explicitly effectful**: Side effects are opt-in and clearly marked
- **Highly testable**: Pure guards and dependency injection via environment traits
- **Type-safe**: Compile-time guarantees about state transitions

## Features

- **Pure Guard Functions**: Deterministic state validation with no side effects
- **Effectful Actions**: Explicit I/O and side effects when needed
- **Environment Pattern**: Clean dependency injection for testing
- **Zero-Cost Abstractions**: Pay only for what you use
- **Composable Effects**: Build complex behavior from simple trait combinations

## Quick Start

### Simple State Machine (Zero Cost)

```rust
use mindset::{StateMachine, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

impl State for ConnectionState {}

fn main() {
    let machine = StateMachine::builder()
        .state(ConnectionState::Disconnected)
        .state(ConnectionState::Connecting)
        .state(ConnectionState::Connected)
        .transition(
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            |_state| true
        )
        .build();

    // Zero-cost transitions - compiles to direct state updates
    machine.transition(ConnectionState::Disconnected, ConnectionState::Connecting);
}
```

### Effectful State Machine

```rust
use mindset::{StateMachine, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OrderState {
    Draft,
    Submitted,
    Processing,
    Completed,
}

impl State for OrderState {}

struct Order {
    id: u64,
    total: f64,
}

// Define environment capabilities as traits
trait PaymentProcessor {
    fn charge(&mut self, amount: f64) -> Result<(), String>;
}

trait Logger {
    fn log(&mut self, message: &str);
}

// Pure guard - no side effects
fn can_submit(order: &Order) -> bool {
    order.total > 0.0
}

// Effectful action - explicit environment usage
fn submit_order<Env>(order: &mut Order, env: &mut Env) -> Result<(), String>
where
    Env: PaymentProcessor + Logger,
{
    env.log(&format!("Submitting order {}", order.id));
    env.charge(order.total)?;
    Ok(())
}

fn main() {
    let machine = StateMachine::builder()
        .state(OrderState::Draft)
        .state(OrderState::Submitted)
        .transition_with_effect(
            OrderState::Draft,
            OrderState::Submitted,
            |order, env| {
                if can_submit(order) {
                    submit_order(order, env)
                } else {
                    Err("Invalid order".to_string())
                }
            }
        )
        .build();

    // Execute with environment
    let mut order = Order { id: 123, total: 99.99 };
    let mut env = ProductionEnv::new();
    machine.transition(&mut order, &mut env).unwrap();
}
```

## Architecture

### Core Design Principles

1. **Pure Guards, Effectful Actions**
   - Guards are pure functions that validate state transitions
   - Actions perform side effects and state changes
   - Clear separation enables testing and reasoning

2. **Zero-Cost by Default**
   - No effects means no runtime overhead
   - Effects are opt-in via explicit environment parameters
   - Compiler optimizes away unused abstractions

3. **Environment Pattern**
   - Dependencies expressed as trait bounds
   - Compose environments from multiple traits
   - Easy mocking for tests

4. **Explicit Over Implicit**
   - Side effects are visible in function signatures
   - No hidden global state or implicit context
   - Clear data flow

### State Machine Model

A state machine in Mindset consists of:

- **States**: Enum variants representing discrete system states
- **Transitions**: Allowed moves between states
- **Guards**: Pure functions determining if transition is allowed
- **Actions**: Effectful functions executed during transition
- **Environment**: External dependencies and services

### Effect-Based Transition Model

Transitions come in two flavors:

#### Pure Transitions (Zero Cost)

```rust
machine.transition(State::A, State::B, |state| {
    // Pure guard logic
    state.is_valid()
});
```

Compiles to direct state updates with no runtime overhead.

#### Effectful Transitions

```rust
machine.transition_with_effect(State::A, State::B, |state, env| {
    // Guard check
    if !state.is_valid() {
        return Err("Invalid state");
    }

    // Effects
    env.log("Transitioning");
    env.save_to_db(state)?;

    // State update
    state.version += 1;

    Ok(())
});
```

Effects are explicit via environment parameter. Only pay for what you use.

## Enforcement System

Mindset includes a validation-based enforcement system that ensures state transitions comply with policies and constraints. Unlike traditional fail-fast error handling, the enforcement system accumulates ALL violations and reports them together, providing comprehensive feedback in a single evaluation.

### Why Validation Over Result?

Traditional `Result` types fail fast - they stop at the first error:

```rust
// With Result: Fix errors one at a time
transition()?;  // Error: Max attempts exceeded
// Fix attempts...
transition()?;  // Error: Timeout exceeded (discovered only after fixing first)
// Fix timeout...
transition()?;  // Error: Custom check failed (discovered only after fixing second)
```

Mindset's `Validation`-based enforcement collects all violations:

```rust
// With Validation: See all errors at once
transition();
// Errors:
//   - Max attempts exceeded (3 max, got 5)
//   - Timeout exceeded (30s max, elapsed 45s)
//   - Custom check failed: Resource unavailable
// Fix everything in one pass
```

### Key Benefits

1. **Complete Feedback**: Users see all violations simultaneously
2. **Better UX**: No frustrating error-fix-error cycles
3. **Stillwater Integration**: Built on proven functional patterns
4. **Flexible Policies**: Combine built-in and custom checks

### Basic Enforcement

```rust
use mindset::enforcement::{EnforcementBuilder, ViolationStrategy};
use std::time::Duration;

let rules = EnforcementBuilder::new()
    .max_attempts(3)
    .timeout(Duration::from_secs(30))
    .on_violation(ViolationStrategy::Abort)
    .build();
```

### Custom Enforcement Checks

Add domain-specific validation:

```rust
let rules = EnforcementBuilder::new()
    .max_attempts(5)
    .require_pred(
        |ctx| ctx.from.is_ready(),
        "Source state must be ready".to_string()
    )
    .require_pred(
        |ctx| !ctx.to.is_locked(),
        "Target state is locked".to_string()
    )
    .on_violation(ViolationStrategy::Retry)
    .build();
```

### Validation Semantics

The enforcement system uses Stillwater's `Validation` type:

- `Validation::Success(())` - All checks passed
- `Validation::Failure(errors)` - Contains ALL violations (using `NonEmptyVec`)

This guarantees that when validation fails, you get a non-empty list of all violations, not just the first one encountered.

### Violation Strategies

Control how violations are handled:

- **Abort** (default): Fail transition permanently
- **Retry**: Allow retry despite violations (for temporary issues)
- **IgnoreAndLog**: Continue with warning (for non-critical checks)

```rust
// Critical safety check - abort on violation
let rules = EnforcementBuilder::new()
    .require_pred(|ctx| ctx.from.is_safe(), "Safety check failed".to_string())
    .on_violation(ViolationStrategy::Abort)
    .build();

// Temporary resource issue - allow retry
let rules = EnforcementBuilder::new()
    .timeout(Duration::from_secs(30))
    .on_violation(ViolationStrategy::Retry)
    .build();
```

For comprehensive documentation on enforcement patterns, custom checks, and best practices, see the [Enforcement Guide](docs/enforcement.md).

## Usage Examples

### Example 1: Traffic Light

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl State for TrafficLight {}

let machine = StateMachine::builder()
    .state(TrafficLight::Red)
    .state(TrafficLight::Yellow)
    .state(TrafficLight::Green)
    .transition(TrafficLight::Red, TrafficLight::Green, |_| true)
    .transition(TrafficLight::Green, TrafficLight::Yellow, |_| true)
    .transition(TrafficLight::Yellow, TrafficLight::Red, |_| true)
    .build();
```

### Example 2: Document Workflow with Logging

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DocState {
    Draft,
    Review,
    Approved,
    Published,
}

impl State for DocState {}

trait AuditLog {
    fn log_transition(&mut self, from: DocState, to: DocState);
}

let machine = StateMachine::builder()
    .state(DocState::Draft)
    .state(DocState::Review)
    .state(DocState::Approved)
    .state(DocState::Published)
    .transition_with_effect(
        DocState::Draft,
        DocState::Review,
        |_, env: &mut impl AuditLog| {
            env.log_transition(DocState::Draft, DocState::Review);
            Ok(())
        }
    )
    .build();
```

### Example 3: State Machine with Validation

```rust
struct Account {
    balance: f64,
    status: AccountStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AccountStatus {
    Active,
    Suspended,
    Closed,
}

impl State for AccountStatus {}

trait AccountRepository {
    fn persist(&mut self, account: &Account) -> Result<(), String>;
}

fn can_close(account: &Account) -> bool {
    account.balance == 0.0
}

fn close_account<Env>(account: &mut Account, env: &mut Env) -> Result<(), String>
where
    Env: AccountRepository,
{
    if !can_close(account) {
        return Err("Cannot close account with non-zero balance".to_string());
    }

    account.status = AccountStatus::Closed;
    env.persist(account)?;

    Ok(())
}
```

## Testing

The environment pattern makes testing straightforward:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockEnv {
        logged: Vec<String>,
        saved: Vec<String>,
    }

    impl Logger for MockEnv {
        fn log(&mut self, msg: &str) {
            self.logged.push(msg.to_string());
        }
    }

    impl Database for MockEnv {
        fn save(&mut self, data: &str) -> Result<(), String> {
            self.saved.push(data.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_effectful_transition() {
        let mut state = State::Initial;
        let mut env = MockEnv {
            logged: vec![],
            saved: vec![],
        };

        transition(&mut state, &mut env).unwrap();

        assert_eq!(env.logged.len(), 1);
        assert_eq!(env.saved.len(), 1);
    }
}
```

## Performance

### Zero-Cost Pure Transitions

Pure transitions have zero runtime overhead. This code:

```rust
machine.transition(State::A, State::B);
```

Compiles to the same assembly as:

```rust
state = State::B;
```

### Effect Cost Model

Effects only cost what you use:

- **No effects**: Zero overhead (direct state update)
- **Single trait**: Monomorphized static dispatch (zero-cost abstraction)
- **Multiple traits**: One vtable lookup per trait (if using trait objects)
- **Environment mutation**: Direct field access

### Benchmarks

On a typical modern CPU:

- Pure transition: ~0.5ns (equivalent to direct assignment)
- Single-effect transition: ~2-3ns (includes function call overhead)
- Multi-effect transition: ~5-10ns (depends on effect complexity)

## Documentation

- [Effects Guide](docs/effects-guide.md): Comprehensive guide to effect patterns
- [Enforcement Guide](docs/enforcement.md): Validation-based policy enforcement
- [API Documentation](https://docs.rs/mindset): Generated API docs

## Design Philosophy

### Functional Core, Imperative Shell

- **Pure core**: Business logic and guards are pure functions
- **Effectful shell**: I/O and side effects at the boundaries
- Clear separation enables testing and reasoning

### Pay Only for What You Use

- Zero-cost when effects aren't needed
- Explicit opt-in for effects
- No hidden overhead or runtime costs

### Explicit Over Implicit

- Side effects visible in function signatures
- Environment dependencies declared as trait bounds
- No magic or hidden behavior

### Testability First

- Pure functions are trivial to test
- Mock environments for integration testing
- Clear dependency injection

## Project Status

This library implements the effect-based state machine foundation as specified in:

- **Spec 001**: Core state machine with pure guards
- **Spec 002**: Effect-based transitions with environment pattern
- **Spec 003**: Validation-based enforcement system

## Contributing

Contributions are welcome! Please ensure:

- All tests pass
- Code follows project conventions
- Documentation is updated
- Commit messages are clear and descriptive

## License

[License information to be added]

## Further Reading

- [Stillwater 0.11.0 Documentation](https://docs.rs/stillwater)
- [Effects Guide](docs/effects-guide.md)
- [State Machine Patterns](docs/patterns.md)
