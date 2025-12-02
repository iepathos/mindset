# Mindset Examples

This directory contains runnable examples demonstrating mindset's state machine patterns.

## Running Examples

Run any example with `cargo run --example <name>`:

```bash
cargo run --example basic_state_machine
cargo run --example document_workflow
cargo run --example checkpoint_resume
```

## Examples Overview

| Example | Demonstrates |
|---------|--------------|
| [basic_state_machine](./basic_state_machine.rs) | Zero-cost state machine with pure transitions |
| [effectful_state_machine](./effectful_state_machine.rs) | Environment pattern and effectful actions |
| [validation_enforcement](./validation_enforcement.rs) | Enforcement rules and validation |
| [testing_patterns](./testing_patterns.rs) | Testing with mock environments |
| [traffic_light](./traffic_light.rs) | Simple cyclic state machine |
| [document_workflow](./document_workflow.rs) | Multi-stage approval workflow |
| [order_processing](./order_processing.rs) | E-commerce order lifecycle |
| [account_management](./account_management.rs) | Account states with validation |
| [checkpoint_resume](./checkpoint_resume.rs) | Checkpoint and resume patterns |
| [mapreduce_workflow](./mapreduce_workflow.rs) | MapReduce workflow implementation |
| [resource_management](./resource_management.rs) | Resource lifecycle management |

## Example Categories

### Foundation Examples

These examples demonstrate the core concepts and zero-cost abstractions:

- **basic_state_machine**: Start here to understand pure state machines with no runtime overhead
- **effectful_state_machine**: Learn the environment pattern for dependency injection and effects
- **testing_patterns**: Discover how to test pure guards and mock environments

### Validation Examples

Learn about the validation-based enforcement system:

- **validation_enforcement**: Comprehensive error reporting with validation over Result

### Real-World Workflows

Practical state machine patterns for common scenarios:

- **traffic_light**: Simple cyclic state transitions
- **document_workflow**: Multi-stage approval process with guards and actions
- **order_processing**: E-commerce order lifecycle with payment and shipping
- **account_management**: Account states with balance and violation validation

### Advanced Patterns

Complex workflows and advanced features:

- **checkpoint_resume**: Long-running workflows with interruption recovery
- **mapreduce_workflow**: Multi-phase workflow orchestration
- **resource_management**: RAII-like resource acquisition and release

## Key Concepts

### Zero-Cost Abstractions

Mindset provides zero-cost state machines when effects aren't needed. Pure transitions compile to simple state updates with no runtime overhead:

```rust
// This compiles to a direct state update
machine.transition(State::A, State::B);
// Equivalent to: state = State::B;
```

### Environment Pattern

Effects are explicit via environment traits. This enables:
- Clean dependency injection
- Easy mocking for tests
- Clear separation of pure logic from side effects

```rust
trait PaymentProcessor {
    fn charge(&mut self, amount: f64) -> Result<(), String>;
}

fn submit_order<Env>(order: &Order, env: &mut Env) -> Result<(), String>
where
    Env: PaymentProcessor,
{
    env.charge(order.total)?;
    Ok(())
}
```

### Pure Guards, Effectful Actions

Guards are pure functions that validate transitions:
```rust
fn can_submit(order: &Order) -> bool {
    order.total > 0.0
}
```

Actions perform side effects and are explicit about their dependencies:
```rust
fn submit_order<Env>(order: &Order, env: &mut Env) -> Result<(), String>
where
    Env: PaymentProcessor + Logger,
{
    env.log("Submitting order");
    env.charge(order.total)?;
    Ok(())
}
```

### Validation Over Result

The enforcement system uses `Validation` instead of `Result` to collect ALL violations:

```rust
// Traditional Result: One error at a time
transition()?;  // Error: Max attempts exceeded
// Fix first error...
transition()?;  // Error: Timeout exceeded (discovered only after fixing first)

// Validation: All errors at once
transition();
// Errors:
//   - Max attempts exceeded (3 max, got 5)
//   - Timeout exceeded (30s max, elapsed 45s)
//   - Custom check failed: Resource unavailable
```

## Learning Path

1. **Start with basics**: Run `basic_state_machine` and `effectful_state_machine`
2. **Understand testing**: Run `testing_patterns` to see how mocking works
3. **Explore validation**: Run `validation_enforcement` to see comprehensive error reporting
4. **Real-world patterns**: Try `document_workflow` and `order_processing`
5. **Advanced features**: Experiment with `checkpoint_resume` and `mapreduce_workflow`

## Documentation

For detailed documentation, see:
- [Main README](../README.md)
- [Builder Guide](../docs/builder-guide.md)
- [Effects Guide](../docs/effects-guide.md)
- [Enforcement Guide](../docs/enforcement.md)
- [Checkpointing Guide](../docs/checkpointing.md)

## Contributing

When adding new examples:
1. Follow the example template structure (see existing examples)
2. Include clear doc comments explaining what the example demonstrates
3. Add the example to this README's table
4. Test that the example compiles and runs without warnings
5. Use realistic scenarios that demonstrate practical patterns
