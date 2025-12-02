---
number: 003
title: Enforcement Observability and Testing Enhancements
category: testing
priority: medium
status: draft
dependencies: [001, 002]
created: 2025-12-01
---

# Specification 003: Enforcement Observability and Testing Enhancements

**Category**: testing
**Priority**: medium
**Status**: draft
**Dependencies**: [001 - Enforcement Integration, 002 - Documentation and Cloneable Enforcement]

## Context

After implementing enforcement integration (Spec 001) and documentation improvements (Spec 002), the library would benefit from enhanced observability and testing capabilities for enforcement behavior.

Currently:
- No telemetry or metrics for enforcement violations
- Limited property-based testing for enforcement accumulation
- No visibility into enforcement decisions during runtime
- Difficult to debug enforcement issues in production
- No way to track enforcement effectiveness over time

These enhancements are not critical for core functionality but significantly improve developer experience, debuggability, and confidence in enforcement correctness.

## Objective

Add comprehensive observability and testing infrastructure for enforcement:
1. Telemetry/metrics for enforcement violations
2. Enhanced property-based tests for violation accumulation
3. Detailed logging of enforcement decisions
4. Documentation of enforcement integration patterns
5. Debugging tools for enforcement issues

## Requirements

### Functional Requirements

- FR1: Add telemetry traits for enforcement events (violations, retries, aborts)
- FR2: Implement optional logging callback for enforcement decisions
- FR3: Add enforcement statistics tracking (violations by type, retry counts)
- FR4: Create property-based tests verifying violation accumulation invariants
- FR5: Add property tests for enforcement timing and attempt counting
- FR6: Document enforcement integration in effects-guide.md
- FR7: Add debugging example showing enforcement observability
- FR8: Support custom enforcement observers via trait

### Non-Functional Requirements

- NFR1: Telemetry must be zero-cost when disabled (compile-time feature flag)
- NFR2: Property tests must run in <5 seconds
- NFR3: Logging must not impact performance (async/buffered)
- NFR4: Observability must work with all violation strategies
- NFR5: Must not require breaking API changes

## Acceptance Criteria

- [ ] `EnforcementObserver` trait defined for custom telemetry
- [ ] `EnforcementStats` struct tracks violation metrics
- [ ] `StateMachine` optionally accepts observer for enforcement events
- [ ] Observer is called on: check start, violations found, strategy applied
- [ ] Default observer logs to structured format (JSON or key=value)
- [ ] Property test: Multiple violations always accumulated together
- [ ] Property test: Attempt counter increments correctly on retry
- [ ] Property test: Timeout enforcement is monotonic with elapsed time
- [ ] Property test: Violation strategies are deterministic
- [ ] Example: `examples/enforcement_debugging.rs` shows observability
- [ ] Example demonstrates violation tracking and statistics
- [ ] effects-guide.md documents enforcement integration patterns
- [ ] Documentation shows how to implement custom observers
- [ ] Feature flag `telemetry` controls observability overhead
- [ ] All tests pass with telemetry enabled and disabled
- [ ] Benchmark shows <1% overhead with telemetry disabled

## Technical Details

### Implementation Approach

#### 1. Enforcement Observer Trait

**File**: `src/enforcement/observer.rs`

```rust
use crate::core::State;
use crate::enforcement::{TransitionContext, ViolationError, ViolationStrategy};
use stillwater::NonEmptyVec;

/// Observer trait for enforcement events.
/// Implement this trait to add custom telemetry, logging, or metrics.
pub trait EnforcementObserver<S: State>: Send + Sync {
    /// Called before enforcement checks are performed.
    fn on_check_start(&self, context: &TransitionContext<S>);

    /// Called when violations are detected.
    fn on_violations(
        &self,
        context: &TransitionContext<S>,
        violations: &NonEmptyVec<ViolationError>,
        strategy: ViolationStrategy,
    );

    /// Called when enforcement passes.
    fn on_check_passed(&self, context: &TransitionContext<S>);

    /// Called when a retry is triggered.
    fn on_retry(&self, context: &TransitionContext<S>, attempt: usize);

    /// Called when transition is aborted due to violations.
    fn on_abort(&self, context: &TransitionContext<S>, violations: &NonEmptyVec<ViolationError>);
}

/// No-op observer (zero cost).
pub struct NoOpObserver;

impl<S: State> EnforcementObserver<S> for NoOpObserver {
    fn on_check_start(&self, _: &TransitionContext<S>) {}
    fn on_violations(&self, _: &TransitionContext<S>, _: &NonEmptyVec<ViolationError>, _: ViolationStrategy) {}
    fn on_check_passed(&self, _: &TransitionContext<S>) {}
    fn on_retry(&self, _: &TransitionContext<S>, _: usize) {}
    fn on_abort(&self, _: &TransitionContext<S>, _: &NonEmptyVec<ViolationError>) {}
}

/// Logging observer that writes to stderr.
pub struct LoggingObserver {
    format: LogFormat,
}

pub enum LogFormat {
    Json,
    KeyValue,
    Human,
}

impl LoggingObserver {
    pub fn new(format: LogFormat) -> Self {
        Self { format }
    }
}

impl<S: State> EnforcementObserver<S> for LoggingObserver {
    fn on_check_start(&self, context: &TransitionContext<S>) {
        match self.format {
            LogFormat::Json => {
                eprintln!(
                    r#"{{"event":"enforcement.check_start","from":"{}","to":"{}","attempt":{}}}"#,
                    context.from.name(),
                    context.to.name(),
                    context.attempt
                );
            }
            LogFormat::KeyValue => {
                eprintln!(
                    "event=enforcement.check_start from={} to={} attempt={}",
                    context.from.name(),
                    context.to.name(),
                    context.attempt
                );
            }
            LogFormat::Human => {
                eprintln!(
                    "[ENFORCEMENT] Checking transition {} -> {} (attempt {})",
                    context.from.name(),
                    context.to.name(),
                    context.attempt
                );
            }
        }
    }

    fn on_violations(
        &self,
        context: &TransitionContext<S>,
        violations: &NonEmptyVec<ViolationError>,
        strategy: ViolationStrategy,
    ) {
        match self.format {
            LogFormat::Json => {
                eprintln!(
                    r#"{{"event":"enforcement.violations","from":"{}","to":"{}","count":{},"strategy":"{}"}}"#,
                    context.from.name(),
                    context.to.name(),
                    violations.len(),
                    format!("{:?}", strategy)
                );
            }
            LogFormat::KeyValue => {
                eprintln!(
                    "event=enforcement.violations from={} to={} count={} strategy={:?}",
                    context.from.name(),
                    context.to.name(),
                    violations.len(),
                    strategy
                );
                for v in violations.iter() {
                    eprintln!("  violation=\"{}\"", v);
                }
            }
            LogFormat::Human => {
                eprintln!(
                    "[ENFORCEMENT] {} violation(s) found (strategy: {:?})",
                    violations.len(),
                    strategy
                );
                for v in violations.iter() {
                    eprintln!("  - {}", v);
                }
            }
        }
    }

    // ... implement other methods
}

/// Statistics-collecting observer.
#[derive(Debug, Clone)]
pub struct StatsObserver {
    stats: Arc<Mutex<EnforcementStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct EnforcementStats {
    pub total_checks: usize,
    pub total_violations: usize,
    pub violations_by_type: HashMap<String, usize>,
    pub retries: usize,
    pub aborts: usize,
}

impl StatsObserver {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(EnforcementStats::default())),
        }
    }

    pub fn stats(&self) -> EnforcementStats {
        self.stats.lock().unwrap().clone()
    }
}

impl<S: State> EnforcementObserver<S> for StatsObserver {
    fn on_check_start(&self, _: &TransitionContext<S>) {
        self.stats.lock().unwrap().total_checks += 1;
    }

    fn on_violations(
        &self,
        _: &TransitionContext<S>,
        violations: &NonEmptyVec<ViolationError>,
        _: ViolationStrategy,
    ) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_violations += violations.len();

        for v in violations.iter() {
            let key = match v {
                ViolationError::MaxAttemptsExceeded { .. } => "max_attempts",
                ViolationError::TimeoutExceeded { .. } => "timeout",
                ViolationError::CustomCheckFailed { .. } => "custom",
            };
            *stats.violations_by_type.entry(key.to_string()).or_insert(0) += 1;
        }
    }

    fn on_retry(&self, _: &TransitionContext<S>, _: usize) {
        self.stats.lock().unwrap().retries += 1;
    }

    fn on_abort(&self, _: &TransitionContext<S>, _: &NonEmptyVec<ViolationError>) {
        self.stats.lock().unwrap().aborts += 1;
    }

    // ... implement other methods
}
```

#### 2. Integrate Observer into StateMachine

**Update**: `src/effects/machine.rs`

```rust
pub struct StateMachine<S: State + 'static, Env: Clone + Send + Sync + 'static> {
    // ... existing fields ...
    #[cfg(feature = "telemetry")]
    enforcement_observer: Option<Arc<dyn EnforcementObserver<S>>>,
}

impl<S: State + 'static, Env: Clone + Send + Sync + 'static> StateMachine<S, Env> {
    pub fn new(initial: S) -> Self {
        Self {
            // ... existing fields ...
            #[cfg(feature = "telemetry")]
            enforcement_observer: None,
        }
    }

    #[cfg(feature = "telemetry")]
    pub fn with_observer(mut self, observer: impl EnforcementObserver<S> + 'static) -> Self {
        self.enforcement_observer = Some(Arc::new(observer));
        self
    }

    pub fn step(&mut self) -> impl Effect<...> {
        // ... existing code ...

        if let Some(enforcement) = &transition.enforcement {
            let context = TransitionContext { /* ... */ };

            #[cfg(feature = "telemetry")]
            if let Some(observer) = &self.enforcement_observer {
                observer.on_check_start(&context);
            }

            match enforcement.enforce(&context) {
                Validation::Failure(errors) => {
                    #[cfg(feature = "telemetry")]
                    if let Some(observer) = &self.enforcement_observer {
                        observer.on_violations(&context, &errors, enforcement.violation_strategy());
                    }

                    // Handle violations...
                }
                Validation::Success(_) => {
                    #[cfg(feature = "telemetry")]
                    if let Some(observer) = &self.enforcement_observer {
                        observer.on_check_passed(&context);
                    }
                }
            }
        }

        // ... rest of step()
    }
}
```

#### 3. Property-Based Tests

**File**: `tests/property_tests.rs` (add to existing)

```rust
use proptest::prelude::*;

proptest! {
    /// Property: If multiple enforcement checks fail, ALL must be reported together.
    #[test]
    fn enforcement_accumulates_all_violations(
        max_attempts in 1usize..10,
        timeout_secs in 1u64..60,
        attempt in 10usize..20,
        elapsed_secs in 61u64..120
    ) {
        let rules = EnforcementBuilder::new()
            .max_attempts(max_attempts)
            .timeout(Duration::from_secs(timeout_secs))
            .require_pred(|_| false, "Custom check fails".to_string())
            .build();

        let context = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt,
            started_at: Utc::now() - chrono::Duration::seconds(elapsed_secs as i64),
        };

        match rules.enforce(&context) {
            Validation::Failure(errors) => {
                // All three checks should fail
                prop_assert_eq!(errors.len(), 3);

                let has_attempts = errors.iter().any(|e| matches!(e, ViolationError::MaxAttemptsExceeded { .. }));
                let has_timeout = errors.iter().any(|e| matches!(e, ViolationError::TimeoutExceeded { .. }));
                let has_custom = errors.iter().any(|e| matches!(e, ViolationError::CustomCheckFailed { .. }));

                prop_assert!(has_attempts);
                prop_assert!(has_timeout);
                prop_assert!(has_custom);
            }
            Validation::Success(_) => {
                return Err(TestCaseError::fail("Expected violations, got success"));
            }
        }
    }

    /// Property: Attempt counter is monotonically increasing on retry.
    #[test]
    fn retry_increments_attempt_counter(
        initial_attempts in 0usize..100
    ) {
        let mut machine = create_test_machine_with_retry_enforcement();
        machine.attempt_count = initial_attempts;

        let result = run_step_sync(&machine);

        match result {
            StepResult::Retry { attempts, .. } => {
                prop_assert_eq!(attempts, initial_attempts + 1);
            }
            _ => {
                // May succeed or abort depending on enforcement config
            }
        }
    }

    /// Property: Timeout enforcement is monotonic with elapsed time.
    #[test]
    fn timeout_monotonic_with_elapsed(
        timeout_secs in 10u64..60,
        elapsed_secs in 0u64..120
    ) {
        let rules = EnforcementBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .build();

        let context = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 1,
            started_at: Utc::now() - chrono::Duration::seconds(elapsed_secs as i64),
        };

        let result = rules.enforce(&context);

        if elapsed_secs > timeout_secs {
            // Must be violation
            prop_assert!(matches!(result, Validation::Failure(_)));
        } else {
            // Must be success
            prop_assert!(matches!(result, Validation::Success(_)));
        }
    }

    /// Property: Violation strategies are deterministic.
    #[test]
    fn violation_strategies_deterministic(
        strategy in prop_oneof![
            Just(ViolationStrategy::Abort),
            Just(ViolationStrategy::Retry),
            Just(ViolationStrategy::IgnoreAndLog)
        ],
        attempt in 0usize..10
    ) {
        let rules = EnforcementBuilder::new()
            .max_attempts(3)
            .on_violation(strategy)
            .build();

        let context = TransitionContext {
            from: TestState::Initial,
            to: TestState::Processing,
            attempt: 5, // Exceeds max
            started_at: Utc::now(),
        };

        // Run twice with same inputs
        let result1 = rules.enforce(&context);
        let result2 = rules.enforce(&context);

        // Must be identical
        match (result1, result2) {
            (Validation::Success(_), Validation::Success(_)) => {},
            (Validation::Failure(e1), Validation::Failure(e2)) => {
                prop_assert_eq!(e1.len(), e2.len());
            }
            _ => return Err(TestCaseError::fail("Non-deterministic enforcement")),
        }

        // Strategy must be consistent
        prop_assert_eq!(rules.violation_strategy(), strategy);
    }
}
```

#### 4. Debugging Example

**File**: `examples/enforcement_debugging.rs`

```rust
//! Enforcement Debugging
//!
//! Demonstrates observability tools for enforcement behavior.
//! Shows how to track violations, retries, and enforcement statistics.

use mindset::{
    builder::{StateMachineBuilder, TransitionBuilder},
    enforcement::{EnforcementBuilder, ViolationStrategy},
    state_enum,
};
use std::time::Duration;

#[cfg(feature = "telemetry")]
use mindset::enforcement::{LoggingObserver, LogFormat, StatsObserver};

state_enum! {
    enum TaskState {
        Pending,
        Running,
        Complete,
        Failed,
    }
    final: [Complete, Failed]
    error: [Failed]
}

#[tokio::main]
async fn main() {
    println!("=== Enforcement Debugging Example ===\n");

    #[cfg(not(feature = "telemetry"))]
    {
        println!("NOTE: Telemetry feature not enabled.");
        println!("Run with: cargo run --example enforcement_debugging --features telemetry\n");
    }

    // Create machine with enforcement and observability
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
                        .timeout(Duration::from_secs(5))
                        .require_pred(
                            |ctx| ctx.attempt < 2,
                            "Attempt must be less than 2".to_string()
                        )
                        .on_violation(ViolationStrategy::Retry)
                        .build()
                )
                .build()
                .unwrap()
        )
        .build()
        .unwrap();

    #[cfg(feature = "telemetry")]
    {
        // Add logging observer
        let logger = LoggingObserver::new(LogFormat::Human);
        let stats = StatsObserver::new();

        machine = machine
            .with_observer(logger)
            .with_observer(stats.clone());

        // Execute multiple attempts (will trigger violations)
        println!("Executing transitions with enforcement...\n");

        let env = ();
        for i in 0..5 {
            println!("--- Attempt {} ---", i + 1);

            match machine.step().run(&env).await {
                Ok((from, result, attempt)) => {
                    machine.apply_result(from, result.clone(), attempt);

                    match result {
                        StepResult::Transitioned(state) => {
                            println!("✓ Transitioned to {:?}\n", state);
                            break;
                        }
                        StepResult::Retry { feedback, attempts } => {
                            println!("⟳ Retry {}: {}\n", attempts, feedback);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        StepResult::Aborted { reason, .. } => {
                            println!("✗ Aborted: {}\n", reason);
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Error: {}\n", e);
                    break;
                }
            }
        }

        // Print statistics
        println!("=== Enforcement Statistics ===");
        let s = stats.stats();
        println!("Total checks: {}", s.total_checks);
        println!("Total violations: {}", s.total_violations);
        println!("Retries: {}", s.retries);
        println!("Aborts: {}", s.aborts);
        println!("\nViolations by type:");
        for (vtype, count) in s.violations_by_type.iter() {
            println!("  {}: {}", vtype, count);
        }
    }

    println!("\n=== Example Complete ===");
}
```

#### 5. Update Effects Guide

Add to `docs/effects-guide.md`:

```markdown
## Enforcement Integration Patterns

### Basic Enforcement Flow

Enforcement checks occur before transition actions:

1. State machine finds applicable transition
2. Creates `TransitionContext` with current state and timing
3. Calls `enforcement.enforce(context)` if rules present
4. Accumulates ALL violations using `Validation::all_vec`
5. Handles violations based on strategy:
   - **Abort**: Returns error, prevents action execution
   - **Retry**: Returns retry result, increments attempt counter
   - **IgnoreAndLog**: Logs warning, proceeds with action
6. Executes action if enforcement passed

### Observability

Enable telemetry to track enforcement behavior:

```rust
#[cfg(feature = "telemetry")]
{
    use mindset::enforcement::{LoggingObserver, LogFormat};

    let machine = StateMachineBuilder::new()
        .initial(State::Idle)
        // ... transitions ...
        .build()
        .unwrap()
        .with_observer(LoggingObserver::new(LogFormat::Json));

    // All enforcement events are now logged
}
```

### Custom Observers

Implement `EnforcementObserver` for custom telemetry:

```rust
struct MetricsObserver {
    client: MetricsClient,
}

impl<S: State> EnforcementObserver<S> for MetricsObserver {
    fn on_violations(&self, ctx: &TransitionContext<S>, violations: &NonEmptyVec<ViolationError>, _: ViolationStrategy) {
        self.client.increment("enforcement.violations", violations.len() as i64);
        self.client.gauge("enforcement.attempt", ctx.attempt as f64);
    }

    // ... implement other methods
}
```

### Statistics Collection

Track enforcement effectiveness over time:

```rust
#[cfg(feature = "telemetry")]
{
    let stats = StatsObserver::new();
    let machine = machine.with_observer(stats.clone());

    // ... execute transitions ...

    let s = stats.stats();
    println!("Violations: {} / {} checks", s.total_violations, s.total_checks);
    println!("Retry rate: {:.1}%", (s.retries as f64 / s.total_checks as f64) * 100.0);
}
```

### Debugging Enforcement Issues

Use logging observer to debug enforcement behavior:

```rust
#[cfg(feature = "telemetry")]
{
    let logger = LoggingObserver::new(LogFormat::Human);
    let machine = machine.with_observer(logger);

    // See detailed logs of enforcement decisions:
    // [ENFORCEMENT] Checking transition Pending -> Running (attempt 1)
    // [ENFORCEMENT] 2 violation(s) found (strategy: Retry)
    //   - Max attempts exceeded (3 max, got 5)
    //   - Timeout exceeded (30s max, elapsed 45s)
}
```
```

### Architecture Changes

- New module: `src/enforcement/observer.rs`
- `StateMachine` gains optional observer field (feature-gated)
- `Cargo.toml` gains new feature flag: `telemetry`
- Integration points in `step()` and violation handling

### Data Structures

```rust
// New traits and types
pub trait EnforcementObserver<S: State>: Send + Sync { ... }
pub struct NoOpObserver;
pub struct LoggingObserver { ... }
pub struct StatsObserver { ... }
pub struct EnforcementStats { ... }

// Feature flag in Cargo.toml
[features]
telemetry = []
```

### APIs and Interfaces

**New Public APIs**:
- `EnforcementObserver` trait (feature-gated)
- `StateMachine::with_observer()` (feature-gated)
- `LoggingObserver`, `StatsObserver` (feature-gated)
- `EnforcementStats` (feature-gated)

No breaking changes to existing APIs.

## Dependencies

- **Prerequisites**:
  - [001 - Enforcement Integration] - Must have working enforcement
  - [002 - Documentation and Cloneable Enforcement] - Docs foundation
- **Affected Components**:
  - `src/enforcement/observer.rs` - New file
  - `src/enforcement/mod.rs` - Export observer types
  - `src/effects/machine.rs` - Observer integration
  - `docs/effects-guide.md` - Documentation additions
  - `examples/enforcement_debugging.rs` - New file
  - `Cargo.toml` - New feature flag
- **External Dependencies**:
  - `parking_lot` (for Mutex in StatsObserver) - optional
  - `serde_json` (for JSON logging) - already dependency

## Testing Strategy

### Unit Tests

Add to `src/enforcement/observer.rs::tests`:

```rust
#[test]
fn noop_observer_has_no_side_effects() {
    let observer = NoOpObserver;
    let ctx = create_test_context();

    // Should not panic or cause issues
    observer.on_check_start(&ctx);
    observer.on_check_passed(&ctx);
}

#[test]
fn stats_observer_tracks_violations() {
    let observer = StatsObserver::new();
    let ctx = create_test_context();

    observer.on_check_start(&ctx);
    observer.on_violations(&ctx, &create_test_violations(), ViolationStrategy::Abort);

    let stats = observer.stats();
    assert_eq!(stats.total_checks, 1);
    assert_eq!(stats.total_violations, 3);
}

#[test]
fn logging_observer_formats_correctly() {
    // Capture stderr and verify format
    let observer = LoggingObserver::new(LogFormat::Json);
    // ... test JSON output format
}
```

### Property Tests

Add to `tests/property_tests.rs` (see Implementation Approach section above).

### Integration Tests

Add to `tests/enforcement_observability.rs`:

```rust
#[cfg(feature = "telemetry")]
#[tokio::test]
async fn observer_receives_all_events() {
    let observer = MockObserver::new();
    let machine = create_machine_with_enforcement()
        .with_observer(observer.clone());

    // Execute transition
    let env = ();
    machine.step().run(&env).await.unwrap();

    // Verify observer was called
    let events = observer.events();
    assert!(events.contains(&Event::CheckStart));
    assert!(events.contains(&Event::Violations) || events.contains(&Event::CheckPassed));
}

#[cfg(feature = "telemetry")]
#[tokio::test]
async fn multiple_observers_all_notified() {
    let observer1 = StatsObserver::new();
    let observer2 = StatsObserver::new();

    let machine = create_machine()
        .with_observer(observer1.clone())
        .with_observer(observer2.clone());

    // Execute
    machine.step().run(&()).await.unwrap();

    // Both observers should have data
    assert_eq!(observer1.stats().total_checks, 1);
    assert_eq!(observer2.stats().total_checks, 1);
}
```

### Feature Flag Tests

Verify compilation with and without telemetry:
```bash
cargo test
cargo test --features telemetry
cargo test --all-features
```

### Benchmark Tests

Add to `benches/enforcement.rs`:

```rust
#[bench]
fn bench_enforcement_without_telemetry(b: &mut Bencher) {
    let machine = create_machine_with_enforcement();
    b.iter(|| {
        // Execute transition
    });
}

#[cfg(feature = "telemetry")]
#[bench]
fn bench_enforcement_with_noop_observer(b: &mut Bencher) {
    let machine = create_machine_with_enforcement()
        .with_observer(NoOpObserver);
    b.iter(|| {
        // Execute transition
    });
}

#[cfg(feature = "telemetry")]
#[bench]
fn bench_enforcement_with_stats_observer(b: &mut Bencher) {
    let machine = create_machine_with_enforcement()
        .with_observer(StatsObserver::new());
    b.iter(|| {
        // Execute transition
    });
}
```

Target: <1% overhead with telemetry disabled, <5% with NoOpObserver.

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for `EnforcementObserver` trait
- Examples showing custom observer implementation
- Document feature flag requirements
- Performance characteristics of each observer type

### User Documentation

**Update docs/effects-guide.md**:
- Add "Enforcement Integration Patterns" section (see Implementation Approach)
- Show observability examples
- Document feature flag usage
- Performance implications

**Add to docs/enforcement.md**:
```markdown
## Observability

### Telemetry

Enable the `telemetry` feature to track enforcement behavior:

```toml
[dependencies]
mindset = { version = "0.2", features = ["telemetry"] }
```

### Built-in Observers

- `NoOpObserver`: Zero-cost observer (default)
- `LoggingObserver`: Log enforcement events to stderr
- `StatsObserver`: Collect enforcement statistics

### Custom Observers

Implement `EnforcementObserver` for custom telemetry.
```

### Architecture Updates

Add to README.md:

```markdown
## Optional Features

- **telemetry**: Enable enforcement observability and metrics (adds ~1% overhead)
```

## Implementation Notes

### Feature Flag Strategy

Use feature flag to eliminate telemetry code when disabled:

```rust
#[cfg(feature = "telemetry")]
pub mod observer;

#[cfg(feature = "telemetry")]
pub use observer::{EnforcementObserver, LoggingObserver, StatsObserver};
```

This ensures zero overhead when telemetry is not needed.

### Observer Design Pattern

Multiple observers are supported via a composite pattern:

```rust
// Internal implementation
struct CompositeObserver<S: State> {
    observers: Vec<Arc<dyn EnforcementObserver<S>>>,
}

impl<S: State> EnforcementObserver<S> for CompositeObserver<S> {
    fn on_check_start(&self, ctx: &TransitionContext<S>) {
        for observer in &self.observers {
            observer.on_check_start(ctx);
        }
    }
    // ... other methods
}
```

### Performance Considerations

- NoOpObserver should inline to nothing
- Feature flag ensures zero cost when disabled
- Logging observer uses buffered I/O
- StatsObserver uses efficient data structures (HashMap)
- Consider async logging for high-throughput scenarios

### Thread Safety

- Observers must be `Send + Sync`
- StatsObserver uses `Arc<Mutex<EnforcementStats>>`
- Consider lock-free alternatives (e.g., atomic counters) for hotpath

### Error Handling

Observers must not panic:
- Catch and log observer errors
- Don't let observer failures impact enforcement
- Consider observer health checks

## Migration and Compatibility

### Breaking Changes

None. Telemetry is opt-in via feature flag.

### API Compatibility

All new APIs are feature-gated, existing APIs unchanged.

### Behavior Changes

No behavior changes when telemetry is disabled.

### Migration Path

1. Upgrade mindset to new version
2. Optionally enable telemetry feature
3. Add observers as needed
4. No code changes required if not using telemetry

### Upgrade Notes

```markdown
## [0.3.0] - 2025-12-XX

### Added
- Observability: EnforcementObserver trait for custom telemetry
- Built-in observers: NoOpObserver, LoggingObserver, StatsObserver
- Property-based tests for enforcement invariants
- Example: enforcement_debugging showing observability
- Feature flag: `telemetry` for opt-in observability

### Performance
- Zero overhead when telemetry disabled (feature flag)
- <1% overhead with NoOpObserver
- <5% overhead with StatsObserver
```

## Success Metrics

- [ ] All 16 acceptance criteria met
- [ ] Feature flag compiles with and without telemetry
- [ ] Property tests verify enforcement invariants
- [ ] Benchmark shows <1% overhead without telemetry
- [ ] Example demonstrates realistic observability use case
- [ ] Documentation covers all observer types
- [ ] All tests pass with all feature combinations
- [ ] Observer trait enables custom telemetry implementations
