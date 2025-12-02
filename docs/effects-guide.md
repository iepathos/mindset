# Effects Guide

This guide explains how to use Stillwater 0.11.0's effect system to build effectful state machines with zero-cost abstractions.

## Overview

Stillwater provides an effect-based transition model that separates pure guard logic from effectful actions. This enables:

- **Pure guards**: Deterministic state validation with no side effects
- **Effectful actions**: Explicit I/O and side effects in transitions
- **Zero-cost abstractions**: No runtime overhead when effects aren't needed
- **Dependency injection**: Clean environment passing for testability

## Core Concepts

### Pure Guards vs Effectful Actions

Guards determine whether a transition is allowed. They should be pure functions:

```rust
fn can_transition(state: &State) -> bool {
    state.is_valid() && state.count > 0
}
```

Actions perform side effects during transitions. They use the effect system:

```rust
fn perform_action<Env>(state: &mut State, env: &mut Env) -> Result<(), Error>
where
    Env: Logger + Database,
{
    env.log("Transitioning state");
    env.save_to_db(state)?;
    state.count += 1;
    Ok(())
}
```

### Zero-Cost Effect System

When you don't need effects, there's zero runtime overhead:

```rust
// No effects - compiles to zero-cost state transitions
let machine = StateMachine::builder()
    .state(Idle)
    .transition(Idle, Running, |_state| true)
    .build();
```

With effects, you pay only for what you use:

```rust
// Effectful transitions - explicit environment threading
let machine = StateMachine::builder()
    .state(Idle)
    .transition_with_effect(Idle, Running, |state, env| {
        env.log("Starting");
        Ok(())
    })
    .build();

// Execute with environment
machine.execute(&mut state, &mut env)?;
```

## Effect Patterns

### Environment Trait Pattern

Define your environment's capabilities as traits:

```rust
pub trait Logger {
    fn log(&mut self, message: &str);
}

pub trait Database {
    fn save(&mut self, data: &str) -> Result<(), Error>;
}

pub trait FileSystem {
    fn read_file(&self, path: &str) -> Result<String, Error>;
    fn write_file(&mut self, path: &str, content: &str) -> Result<(), Error>;
}
```

Compose environments from multiple traits:

```rust
struct AppEnv {
    logger: ConsoleLogger,
    db: SqliteDb,
}

impl Logger for AppEnv {
    fn log(&mut self, message: &str) {
        self.logger.log(message)
    }
}

impl Database for AppEnv {
    fn save(&mut self, data: &str) -> Result<(), Error> {
        self.db.save(data)
    }
}
```

### Effectful Transition Example

Here's a complete example of an effectful state machine:

```rust
use stillwater::{StateMachine, State};

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

trait PaymentGateway {
    fn charge(&mut self, amount: f64) -> Result<(), String>;
}

trait NotificationService {
    fn notify(&mut self, message: &str) -> Result<(), String>;
}

fn submit_order<Env>(order: &mut Order, env: &mut Env) -> Result<(), String>
where
    Env: PaymentGateway + NotificationService,
{
    env.charge(order.total)?;
    env.notify(&format!("Order {} submitted", order.id))?;
    Ok(())
}

fn main() {
    let machine = StateMachine::builder()
        .state(OrderState::Draft)
        .state(OrderState::Submitted)
        .transition_with_effect(
            OrderState::Draft,
            OrderState::Submitted,
            submit_order,
        )
        .build();

    // Use with real environment
    let mut order = Order { id: 123, total: 99.99 };
    let mut env = ProductionEnv::new();
    machine.transition(&mut order, &mut env).unwrap();
}
```

### Testing with Mock Environments

The environment pattern makes testing easy:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockEnv {
        charges: Vec<f64>,
        notifications: Vec<String>,
    }

    impl PaymentGateway for MockEnv {
        fn charge(&mut self, amount: f64) -> Result<(), String> {
            self.charges.push(amount);
            Ok(())
        }
    }

    impl NotificationService for MockEnv {
        fn notify(&mut self, message: &str) -> Result<(), String> {
            self.notifications.push(message.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_order_submission() {
        let mut order = Order { id: 123, total: 99.99 };
        let mut env = MockEnv {
            charges: vec![],
            notifications: vec![],
        };

        submit_order(&mut order, &mut env).unwrap();

        assert_eq!(env.charges.len(), 1);
        assert_eq!(env.charges[0], 99.99);
        assert_eq!(env.notifications.len(), 1);
    }
}
```

## Advanced Patterns

### Conditional Effects

Use guards to decide when effects should run:

```rust
fn maybe_send_notification<Env>(state: &State, env: &mut Env) -> Result<(), Error>
where
    Env: NotificationService,
{
    if state.should_notify {
        env.notify("State changed")?;
    }
    Ok(())
}
```

### Effect Composition

Compose multiple effects in a transition:

```rust
fn complex_transition<Env>(state: &mut State, env: &mut Env) -> Result<(), Error>
where
    Env: Logger + Database + NotificationService,
{
    env.log("Starting transition");

    // Validate
    if !state.is_valid() {
        return Err(Error::InvalidState);
    }

    // Persist
    env.save_to_db(&state.serialize()?)?;

    // Notify
    env.notify("State updated")?;

    // Update state
    state.version += 1;

    Ok(())
}
```

### Error Handling in Effects

Effects return `Result` for proper error handling:

```rust
fn safe_transition<Env>(state: &mut State, env: &mut Env) -> Result<(), Error>
where
    Env: Database,
{
    match env.save_to_db(state) {
        Ok(_) => {
            state.persisted = true;
            Ok(())
        }
        Err(e) => {
            env.log_error(&format!("Failed to save: {}", e));
            Err(Error::PersistenceFailed(e))
        }
    }
}
```

## Best Practices

### Keep Guards Pure

Guards should never have side effects:

```rust
// Good - pure guard
fn can_submit(order: &Order) -> bool {
    order.total > 0.0 && !order.items.is_empty()
}

// Bad - effectful guard
fn can_submit<Env>(order: &Order, env: &mut Env) -> bool {
    env.log("Checking if can submit"); // Side effect!
    order.total > 0.0
}
```

### Use Trait Bounds for Effects

Be explicit about what effects a transition needs:

```rust
// Good - explicit trait bounds
fn process<Env>(state: &mut State, env: &mut Env) -> Result<(), Error>
where
    Env: Database + Logger,
{
    env.log("Processing");
    env.save(state)?;
    Ok(())
}

// Avoid - generic environment without bounds
fn process<Env>(state: &mut State, env: &mut Env) -> Result<(), Error> {
    // What can env do? Unclear!
    Ok(())
}
```

### Separate Business Logic from I/O

Keep state transformations pure, effects at the boundaries:

```rust
// Pure business logic
fn calculate_total(items: &[Item]) -> f64 {
    items.iter().map(|i| i.price).sum()
}

// Effectful boundary
fn save_order<Env>(order: &mut Order, env: &mut Env) -> Result<(), Error>
where
    Env: Database,
{
    order.total = calculate_total(&order.items);
    env.save(order)?;
    Ok(())
}
```

### Use Zero-Cost When Possible

If you don't need effects, don't use them:

```rust
// Simple state machine - no effects needed
let machine = StateMachine::builder()
    .state(State::A)
    .state(State::B)
    .transition(State::A, State::B, |_| true)
    .build();
```

Only add effects when necessary:

```rust
// Now we need logging - add it explicitly
let machine = StateMachine::builder()
    .state(State::A)
    .state(State::B)
    .transition_with_effect(State::A, State::B, |_, env: &mut Logger| {
        env.log("Transitioning");
        Ok(())
    })
    .build();
```

## Performance Characteristics

### Zero-Cost Pure Transitions

Pure transitions compile to direct function calls with no overhead:

```rust
// This has zero runtime cost over manual state updates
machine.transition(State::A, State::B);
```

### Effect Cost Model

Effects only cost what you use:

- **No effects**: Zero overhead
- **Single trait**: One vtable lookup (if dynamic) or monomorphized (if static)
- **Multiple traits**: One lookup per trait
- **Environment mutation**: Direct field access, no allocation

### Optimization Tips

1. **Use static dispatch when possible**: Generic `Env` parameter allows monomorphization
2. **Keep environment types small**: Pass references to services, not large structs
3. **Batch effects**: Combine multiple state changes in one transition
4. **Avoid allocations**: Use references and borrows in effect signatures

## Migration Guide

### From Effectful Guards

If you have guards with side effects:

```rust
// Before
fn guard_with_logging<Env>(state: &State, env: &mut Env) -> bool {
    env.log("Checking guard");
    state.is_valid()
}
```

Split into pure guard and effectful action:

```rust
// After
fn guard(state: &State) -> bool {
    state.is_valid()
}

fn action<Env>(state: &mut State, env: &mut Env) -> Result<(), Error> {
    env.log("Transitioning");
    Ok(())
}
```

### From Monolithic Actions

If you have large effectful functions:

```rust
// Before
fn big_transition<Env>(state: &mut State, env: &mut Env) -> Result<(), Error> {
    env.log("Starting");
    let data = calculate_data(state);
    env.save(data)?;
    state.counter += 1;
    env.notify("Done");
    Ok(())
}
```

Extract pure logic:

```rust
// After
fn calculate_data(state: &State) -> Data {
    // Pure calculation
    Data::from(state)
}

fn transition<Env>(state: &mut State, env: &mut Env) -> Result<(), Error>
where
    Env: Logger + Database + NotificationService,
{
    env.log("Starting");

    let data = calculate_data(state);
    env.save(data)?;

    state.counter += 1;

    env.notify("Done");
    Ok(())
}
```

## Summary

Stillwater's effect system provides:

- **Pure guards**: Deterministic, testable state validation
- **Explicit effects**: Side effects only where needed
- **Zero-cost abstraction**: No overhead when effects aren't used
- **Dependency injection**: Clean environment pattern for testing
- **Composable traits**: Build complex environments from simple pieces

Use effects to keep your state machines testable, maintainable, and performant.
