# Mindset: Type-Safe State Machine Library - Implementation Plan

**Version:** 0.1.0
**Status:** Planning
**Created:** 2025-12-01
**Target:** Production-ready state machine library built on Stillwater

---

## Project Overview

**Mindset** is a pure functional state machine library built on Stillwater's Effect system. It provides type-safe state transitions, enforcement mechanisms, and checkpoint/resume capabilities. It will be used by Platypus for AI workflow orchestration but is designed as a general-purpose library.

### Core Philosophy

- **Pure state transitions** - State changes are pure functions
- **Effect-based actions** - Side effects isolated in Effect monads
- **Type-safe guarantees** - Invalid transitions caught at compile time
- **Enforcement first** - Rules can require verification before transitions
- **Checkpoint-friendly** - Built-in serialization and resume support

### Dependencies

- `stillwater` - Effect system, validation, reader pattern
- `serde` - Serialization for checkpoints
- `chrono` - Timestamps for state history
- `anyhow` - Error handling
- `thiserror` - Custom error types

---

## Architecture

### Core Types

```rust
pub trait State: Clone + PartialEq + Debug + Serialize + DeserializeOwned {
    fn name(&self) -> &str;
    fn is_final(&self) -> bool { false }
    fn is_error(&self) -> bool { false }
}

pub struct Transition<E, S: State> {
    pub from: S,
    pub to: S,
    pub guard: Option<Guard<S>>,           // Pre-condition check
    pub action: Effect<E, TransitionResult<S>>,
    pub enforcement: Option<EnforcementRules>,
}

pub struct StateMachine<E, S: State> {
    initial: S,
    current: S,
    transitions: Vec<Transition<E, S>>,
    history: Vec<StateTransition<S>>,
    metadata: MachineMetadata,
}

pub struct EnforcementRules {
    pub max_attempts: Option<usize>,
    pub timeout: Option<Duration>,
    pub required_checks: Vec<CheckFn>,
    pub on_violation: ViolationStrategy,
}

pub enum TransitionResult<S> {
    Success(S),                    // Transition succeeded
    Retry { feedback: String },    // Retry with context
    Abort { reason: String },      // Permanent failure
}

pub struct StateTransition<S> {
    pub from: S,
    pub to: S,
    pub timestamp: DateTime<Utc>,
    pub attempt: usize,
    pub enforcement_result: Option<EnforcementResult>,
}
```

### Module Structure

```
mindset/
├── Cargo.toml
├── README.md
├── IMPLEMENTATION_PLAN.md
├── src/
│   ├── lib.rs                 # Public API exports
│   ├── state.rs               # State trait and core types
│   ├── transition.rs          # Transition logic
│   ├── machine.rs             # StateMachine implementation
│   ├── enforcement.rs         # Enforcement rules and checking
│   ├── guard.rs               # Pre-condition guards
│   ├── checkpoint.rs          # Serialization/deserialization
│   ├── history.rs             # State history tracking
│   ├── builder.rs             # Fluent builder API
│   └── error.rs               # Error types
└── tests/
    ├── basic_machine_tests.rs
    ├── enforcement_tests.rs
    ├── checkpoint_tests.rs
    └── integration_tests.rs
```

---

## Implementation Stages

### Stage 1: Core State Machine (Foundation)

**Goal:** Implement basic state machine with pure transitions

**Success Criteria:**
- [ ] `State` trait defined
- [ ] `Transition` struct with guard and action
- [ ] `StateMachine` can execute simple transitions
- [ ] State history tracking works
- [ ] All tests pass

**Tasks:**

1. **Setup project structure**
   - Create Cargo project with dependencies
   - Setup module structure
   - Configure dev dependencies (test utilities)

2. **Implement State trait and core types**
   ```rust
   // state.rs
   pub trait State: Clone + PartialEq + Debug {
       fn name(&self) -> &str;
       fn is_final(&self) -> bool;
       fn is_error(&self) -> bool;
   }
   ```

3. **Implement Transition logic**
   ```rust
   // transition.rs
   pub struct Transition<E, S: State> {
       from: S,
       to: S,
       guard: Option<Guard<S>>,
       action: Effect<E, TransitionResult<S>>,
   }

   impl<E, S: State> Transition<E, S> {
       pub fn can_execute(&self, current: &S) -> bool {
           *current == self.from && self.check_guard(current)
       }

       fn check_guard(&self, state: &S) -> bool {
           self.guard.as_ref().map_or(true, |g| g.check(state))
       }
   }
   ```

4. **Implement StateMachine core**
   ```rust
   // machine.rs
   pub struct StateMachine<E, S: State> {
       initial: S,
       current: S,
       transitions: Vec<Transition<E, S>>,
       history: Vec<StateTransition<S>>,
   }

   impl<E, S: State> StateMachine<E, S> {
       pub fn new(initial: S) -> Self;
       pub fn add_transition(&mut self, transition: Transition<E, S>);
       pub fn step(&mut self) -> Effect<E, StepResult<S>>;
       pub fn current_state(&self) -> &S;
       pub fn is_final(&self) -> bool;
   }
   ```

5. **Implement history tracking**
   ```rust
   // history.rs
   pub struct StateTransition<S> {
       from: S,
       to: S,
       timestamp: DateTime<Utc>,
       attempt: usize,
   }

   pub struct StateHistory<S> {
       transitions: Vec<StateTransition<S>>,
   }

   impl<S> StateHistory<S> {
       pub fn record(&mut self, transition: StateTransition<S>);
       pub fn get_path(&self) -> Vec<&S>;
       pub fn duration(&self) -> Option<Duration>;
   }
   ```

**Tests:**
```rust
#[test]
fn simple_transition_succeeds() {
    let machine = StateMachine::new(State::Initial)
        .add_transition(Transition {
            from: State::Initial,
            to: State::Processing,
            guard: None,
            action: Effect::pure(TransitionResult::Success(State::Processing)),
        });

    let result = machine.step().run(&env);
    assert!(result.is_ok());
    assert_eq!(machine.current_state(), &State::Processing);
}

#[test]
fn guard_prevents_invalid_transition() {
    let guard = Guard::new(|state| state.data.is_valid());
    let machine = StateMachine::new(State::Invalid)
        .add_transition(Transition {
            from: State::Invalid,
            to: State::Valid,
            guard: Some(guard),
            action: Effect::pure(TransitionResult::Success(State::Valid)),
        });

    let result = machine.step().run(&env);
    assert!(result.is_err()); // Guard should block
}

#[test]
fn history_tracks_all_transitions() {
    let mut machine = StateMachine::new(State::Initial);
    // ... add transitions and step through states ...

    let history = machine.history();
    assert_eq!(history.len(), 3);
    assert_eq!(history.get_path(), vec![&State::Initial, &State::Processing, &State::Complete]);
}
```

**Status:** Not Started

---

### Stage 2: Enforcement System

**Goal:** Add enforcement rules that can verify state transitions

**Success Criteria:**
- [ ] `EnforcementRules` can be attached to transitions
- [ ] Max attempts enforcement works
- [ ] Timeout enforcement works
- [ ] Custom checks can be defined and executed
- [ ] Violation strategies work (abort, retry, ignore)
- [ ] All enforcement tests pass

**Tasks:**

1. **Define EnforcementRules**
   ```rust
   // enforcement.rs
   pub struct EnforcementRules {
       max_attempts: Option<usize>,
       timeout: Option<Duration>,
       required_checks: Vec<Box<dyn CheckFn>>,
       on_violation: ViolationStrategy,
   }

   pub trait CheckFn: Fn(&TransitionContext) -> Result<(), CheckError> {}

   pub enum ViolationStrategy {
       Abort,           // Fail permanently
       Retry,           // Retry transition
       IgnoreAndLog,    // Continue but log warning
   }
   ```

2. **Implement enforcement checking**
   ```rust
   impl EnforcementRules {
       pub fn enforce(&self, context: &TransitionContext) -> EnforcementResult {
           // Check max attempts
           if let Some(max) = self.max_attempts {
               if context.attempt > max {
                   return EnforcementResult::Violated(ViolationError::MaxAttemptsExceeded);
               }
           }

           // Check timeout
           if let Some(timeout) = self.timeout {
               if context.elapsed() > timeout {
                   return EnforcementResult::Violated(ViolationError::TimeoutExceeded);
               }
           }

           // Run custom checks
           for check in &self.required_checks {
               check(context)?;
           }

           EnforcementResult::Passed
       }
   }
   ```

3. **Integrate enforcement into transitions**
   ```rust
   impl<E, S: State> StateMachine<E, S> {
       pub fn step_with_enforcement(&mut self) -> Effect<E, StepResult<S>> {
           // Find applicable transition
           let transition = self.find_transition()?;

           // Check enforcement rules if present
           if let Some(rules) = &transition.enforcement {
               let context = TransitionContext {
                   from: &self.current,
                   to: &transition.to,
                   attempt: self.get_attempt_count(&transition),
                   started_at: self.transition_start_time,
               };

               match rules.enforce(&context) {
                   EnforcementResult::Passed => {
                       // Proceed with transition
                   }
                   EnforcementResult::Violated(err) => {
                       return self.handle_violation(err, &rules.on_violation);
                   }
               }
           }

           // Execute transition action
           transition.action.clone()
       }
   }
   ```

4. **Add builder API for enforcement rules**
   ```rust
   impl EnforcementRules {
       pub fn builder() -> EnforcementBuilder {
           EnforcementBuilder::new()
       }
   }

   pub struct EnforcementBuilder {
       rules: EnforcementRules,
   }

   impl EnforcementBuilder {
       pub fn max_attempts(mut self, n: usize) -> Self {
           self.rules.max_attempts = Some(n);
           self
       }

       pub fn timeout(mut self, duration: Duration) -> Self {
           self.rules.timeout = Some(duration);
           self
       }

       pub fn require(mut self, check: impl CheckFn + 'static) -> Self {
           self.rules.required_checks.push(Box::new(check));
           self
       }

       pub fn on_violation(mut self, strategy: ViolationStrategy) -> Self {
           self.rules.on_violation = strategy;
           self
       }

       pub fn build(self) -> EnforcementRules {
           self.rules
       }
   }
   ```

**Tests:**
```rust
#[test]
fn max_attempts_enforced() {
    let enforcement = EnforcementRules::builder()
        .max_attempts(3)
        .on_violation(ViolationStrategy::Abort)
        .build();

    let transition = Transition {
        enforcement: Some(enforcement),
        action: Effect::pure(TransitionResult::Retry { feedback: "Try again" }),
        ..
    };

    // Should succeed for attempts 1, 2, 3
    // Should abort on attempt 4
}

#[test]
fn timeout_enforced() {
    let enforcement = EnforcementRules::builder()
        .timeout(Duration::from_secs(5))
        .build();

    // Transition should abort if it takes > 5 seconds
}

#[test]
fn custom_check_can_block_transition() {
    let enforcement = EnforcementRules::builder()
        .require(|ctx| {
            if ctx.from.data.is_valid() {
                Ok(())
            } else {
                Err(CheckError::ValidationFailed)
            }
        })
        .build();

    // Transition should be blocked if check fails
}
```

**Status:** Not Started

---

### Stage 3: Checkpoint & Resume

**Goal:** Enable serialization and resumption of state machines

**Success Criteria:**
- [ ] StateMachine can be serialized to JSON/binary
- [ ] StateMachine can be deserialized and resume from checkpoint
- [ ] History is preserved across checkpoint/resume
- [ ] Metadata (start time, attempt counts) preserved
- [ ] All checkpoint tests pass

**Tasks:**

1. **Add Serialize/Deserialize to core types**
   ```rust
   // State trait requires Serialize + DeserializeOwned
   pub trait State: Clone + PartialEq + Debug + Serialize + DeserializeOwned {
       // ...
   }

   #[derive(Serialize, Deserialize)]
   pub struct StateMachine<E, S: State> {
       // All fields must be serializable
   }
   ```

2. **Implement checkpoint creation**
   ```rust
   // checkpoint.rs
   pub struct Checkpoint<S: State> {
       pub id: String,
       pub timestamp: DateTime<Utc>,
       pub current_state: S,
       pub history: Vec<StateTransition<S>>,
       pub metadata: MachineMetadata,
   }

   impl<E, S: State> StateMachine<E, S> {
       pub fn checkpoint(&self) -> Checkpoint<S> {
           Checkpoint {
               id: Uuid::new_v4().to_string(),
               timestamp: Utc::now(),
               current_state: self.current.clone(),
               history: self.history.clone(),
               metadata: self.metadata.clone(),
           }
       }

       pub fn to_json(&self) -> Result<String> {
           serde_json::to_string_pretty(&self.checkpoint())
       }

       pub fn to_binary(&self) -> Result<Vec<u8>> {
           bincode::serialize(&self.checkpoint())
       }
   }
   ```

3. **Implement resume from checkpoint**
   ```rust
   impl<E, S: State> StateMachine<E, S> {
       pub fn from_checkpoint(checkpoint: Checkpoint<S>, transitions: Vec<Transition<E, S>>) -> Self {
           StateMachine {
               initial: checkpoint.history.first()
                   .map(|t| t.from.clone())
                   .unwrap_or_else(|| checkpoint.current_state.clone()),
               current: checkpoint.current_state,
               transitions,
               history: checkpoint.history,
               metadata: checkpoint.metadata,
           }
       }

       pub fn from_json(json: &str, transitions: Vec<Transition<E, S>>) -> Result<Self> {
           let checkpoint: Checkpoint<S> = serde_json::from_str(json)?;
           Ok(Self::from_checkpoint(checkpoint, transitions))
       }
   }
   ```

4. **Add metadata tracking**
   ```rust
   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub struct MachineMetadata {
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
       pub total_attempts: HashMap<String, usize>,  // transition name -> attempts
       pub total_duration: Duration,
   }
   ```

**Tests:**
```rust
#[test]
fn checkpoint_preserves_state() {
    let mut machine = create_test_machine();
    machine.step().run(&env);
    machine.step().run(&env);

    let checkpoint = machine.checkpoint();
    assert_eq!(checkpoint.current_state, machine.current_state());
    assert_eq!(checkpoint.history.len(), 2);
}

#[test]
fn resume_from_checkpoint_continues_execution() {
    let mut machine1 = create_test_machine();
    machine1.step().run(&env);
    machine1.step().run(&env);

    let json = machine1.to_json().unwrap();

    let mut machine2 = StateMachine::from_json(&json, create_transitions()).unwrap();

    assert_eq!(machine2.current_state(), machine1.current_state());
    assert_eq!(machine2.history().len(), machine1.history().len());

    // Should be able to continue from where we left off
    machine2.step().run(&env);
}

#[test]
fn binary_serialization_roundtrip() {
    let machine1 = create_test_machine();
    let bytes = machine1.to_binary().unwrap();
    let machine2 = StateMachine::from_binary(&bytes, create_transitions()).unwrap();

    assert_eq!(machine1.current_state(), machine2.current_state());
}
```

**Status:** Not Started

---

### Stage 4: Builder API & Ergonomics

**Goal:** Provide fluent, ergonomic API for building state machines

**Success Criteria:**
- [ ] Fluent builder API works
- [ ] Type inference works well
- [ ] Error messages are clear
- [ ] Documentation examples compile
- [ ] API feels natural to use

**Tasks:**

1. **Implement StateMachine builder**
   ```rust
   // builder.rs
   pub struct StateMachineBuilder<E, S: State> {
       initial: Option<S>,
       transitions: Vec<Transition<E, S>>,
       _phantom: PhantomData<E>,
   }

   impl<E, S: State> StateMachineBuilder<E, S> {
       pub fn new() -> Self {
           Self {
               initial: None,
               transitions: vec![],
               _phantom: PhantomData,
           }
       }

       pub fn initial(mut self, state: S) -> Self {
           self.initial = Some(state);
           self
       }

       pub fn transition(mut self, transition: Transition<E, S>) -> Self {
           self.transitions.push(transition);
           self
       }

       pub fn build(self) -> Result<StateMachine<E, S>> {
           let initial = self.initial.ok_or(BuildError::NoInitialState)?;
           Ok(StateMachine::new(initial, self.transitions))
       }
   }
   ```

2. **Implement Transition builder**
   ```rust
   pub struct TransitionBuilder<E, S: State> {
       from: Option<S>,
       to: Option<S>,
       guard: Option<Guard<S>>,
       action: Option<Effect<E, TransitionResult<S>>>,
       enforcement: Option<EnforcementRules>,
   }

   impl<E, S: State> TransitionBuilder<E, S> {
       pub fn new() -> Self;
       pub fn from(mut self, state: S) -> Self;
       pub fn to(mut self, state: S) -> Self;
       pub fn guard(mut self, guard: Guard<S>) -> Self;
       pub fn action(mut self, action: Effect<E, TransitionResult<S>>) -> Self;
       pub fn enforce(mut self, rules: EnforcementRules) -> Self;
       pub fn build(self) -> Result<Transition<E, S>>;
   }
   ```

3. **Add convenience macros**
   ```rust
   // Macro for simple state enums
   #[macro_export]
   macro_rules! state_enum {
       ($name:ident { $($variant:ident),* $(,)? }) => {
           #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
           pub enum $name {
               $($variant),*
           }

           impl State for $name {
               fn name(&self) -> &str {
                   match self {
                       $(Self::$variant => stringify!($variant)),*
                   }
               }
           }
       };
   }

   // Usage:
   // state_enum!(WorkflowState { Initial, Processing, Complete });
   ```

4. **Improve error messages**
   ```rust
   // error.rs
   #[derive(Debug, thiserror::Error)]
   pub enum MachineError {
       #[error("No transition available from state '{from}' to '{to}'")]
       NoTransition { from: String, to: String },

       #[error("Guard prevented transition from '{from}' to '{to}': {reason}")]
       GuardBlocked { from: String, to: String, reason: String },

       #[error("Enforcement violation: {0}")]
       EnforcementViolation(#[from] ViolationError),

       #[error("State machine is stuck in non-final state '{state}'")]
       Stuck { state: String },
   }
   ```

**Tests:**
```rust
#[test]
fn builder_api_is_ergonomic() {
    let machine = StateMachineBuilder::new()
        .initial(State::Initial)
        .transition(
            TransitionBuilder::new()
                .from(State::Initial)
                .to(State::Processing)
                .action(Effect::pure(TransitionResult::Success(State::Processing)))
                .build()
                .unwrap()
        )
        .transition(
            TransitionBuilder::new()
                .from(State::Processing)
                .to(State::Complete)
                .enforce(
                    EnforcementRules::builder()
                        .max_attempts(3)
                        .build()
                )
                .action(Effect::pure(TransitionResult::Success(State::Complete)))
                .build()
                .unwrap()
        )
        .build()
        .unwrap();

    assert_eq!(machine.current_state(), &State::Initial);
}

#[test]
fn state_enum_macro_works() {
    state_enum!(TestState { Start, Middle, End });

    let state = TestState::Start;
    assert_eq!(state.name(), "Start");
    assert!(!state.is_final());
}
```

**Status:** Not Started

---

### Stage 5: Advanced Features & Polish

**Goal:** Add run_until_final, visualization, and production-ready features

**Success Criteria:**
- [ ] `run_until_final()` executes complete workflows
- [ ] DOT graph generation works
- [ ] Error handling is comprehensive
- [ ] Documentation is complete
- [ ] Performance is acceptable
- [ ] Ready for 0.1.0 release

**Tasks:**

1. **Implement run_until_final**
   ```rust
   impl<E, S: State> StateMachine<E, S> {
       pub fn run_until_final(&mut self) -> Effect<E, S> {
           self.step()
               .and_then(|result| {
                   match result {
                       StepResult::Transitioned(state) if state.is_final() => {
                           Effect::pure(state)
                       }
                       StepResult::Transitioned(_) => {
                           self.run_until_final()
                       }
                       StepResult::Retry(feedback) => {
                           // Log feedback and retry
                           self.run_until_final()
                       }
                       StepResult::NoTransition => {
                           Effect::fail(MachineError::Stuck {
                               state: self.current.name().to_string()
                           })
                       }
                   }
               })
       }
   }
   ```

2. **Add DOT graph visualization**
   ```rust
   // visualization.rs
   impl<E, S: State> StateMachine<E, S> {
       pub fn to_dot(&self) -> String {
           let mut dot = String::from("digraph StateMachine {\n");
           dot.push_str("  rankdir=LR;\n");

           // Add states
           for state in self.all_states() {
               let shape = if state.is_final() { "doublecircle" } else { "circle" };
               dot.push_str(&format!("  {} [shape={}];\n", state.name(), shape));
           }

           // Add transitions
           for transition in &self.transitions {
               let label = if transition.enforcement.is_some() {
                   "[enforced]"
               } else {
                   ""
               };
               dot.push_str(&format!(
                   "  {} -> {} [label=\"{}\"];\n",
                   transition.from.name(),
                   transition.to.name(),
                   label
               ));
           }

           dot.push_str("}\n");
           dot
       }
   }
   ```

3. **Add observability hooks**
   ```rust
   pub trait MachineObserver<S: State> {
       fn on_transition_start(&self, from: &S, to: &S);
       fn on_transition_complete(&self, from: &S, to: &S, duration: Duration);
       fn on_transition_failed(&self, from: &S, to: &S, error: &MachineError);
       fn on_enforcement_violation(&self, violation: &ViolationError);
   }

   impl<E, S: State> StateMachine<E, S> {
       pub fn with_observer(mut self, observer: Box<dyn MachineObserver<S>>) -> Self {
           self.observer = Some(observer);
           self
       }
   }
   ```

4. **Performance optimization**
   - Use `HashMap` for O(1) transition lookup
   - Lazy evaluation where possible
   - Minimize cloning with `Rc`/`Arc` where appropriate
   - Benchmark critical paths

5. **Complete documentation**
   - API documentation with examples
   - README with getting started guide
   - Architecture documentation
   - Migration guide (if needed)

**Tests:**
```rust
#[test]
fn run_until_final_completes_workflow() {
    let mut machine = create_multi_step_machine();
    let final_state = machine.run_until_final().run(&env).unwrap();

    assert!(final_state.is_final());
    assert_eq!(machine.history().len(), 5); // All steps executed
}

#[test]
fn dot_graph_generation_works() {
    let machine = create_test_machine();
    let dot = machine.to_dot();

    assert!(dot.contains("digraph StateMachine"));
    assert!(dot.contains("Initial -> Processing"));
    assert!(dot.contains("Processing -> Complete"));
}

#[test]
fn observer_receives_all_events() {
    let observer = MockObserver::new();
    let mut machine = create_test_machine().with_observer(Box::new(observer.clone()));

    machine.step().run(&env);

    assert_eq!(observer.transition_starts(), 1);
    assert_eq!(observer.transition_completes(), 1);
}
```

**Status:** Not Started

---

## Testing Strategy

### Unit Tests
- Each module has comprehensive unit tests
- Test pure functions in isolation
- Mock effects for testing

### Integration Tests
- Multi-step state machine workflows
- Checkpoint/resume scenarios
- Enforcement rule combinations
- Error handling paths

### Property-Based Tests
```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn checkpoint_roundtrip_preserves_state(
            state in arbitrary_state(),
            history in arbitrary_history()
        ) {
            let machine1 = create_machine_with_state(state, history);
            let json = machine1.to_json().unwrap();
            let machine2 = StateMachine::from_json(&json, transitions()).unwrap();

            prop_assert_eq!(machine1.current_state(), machine2.current_state());
        }
    }
}
```

### Benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_transition(c: &mut Criterion) {
    c.bench_function("simple_transition", |b| {
        b.iter(|| {
            let mut machine = create_simple_machine();
            machine.step().run(&env)
        });
    });
}

criterion_group!(benches, benchmark_transition);
criterion_main!(benches);
```

---

## Documentation Requirements

### README.md
- Quick start example
- Installation instructions
- Basic usage examples
- Link to full documentation

### API Documentation
- All public types documented
- Examples for common patterns
- Links between related types

### Architecture Guide
- Design decisions explained
- How Effect integration works
- Extension points

### Examples
- Simple state machine
- State machine with enforcement
- Checkpoint/resume workflow
- Custom guards and checks

---

## Success Metrics

### Correctness
- [ ] 100% of unit tests pass
- [ ] 100% of integration tests pass
- [ ] Property tests find no violations
- [ ] No memory leaks (valgrind/miri)

### Performance
- [ ] Simple transition < 1μs
- [ ] Checkpoint creation < 100μs
- [ ] JSON serialization < 1ms for typical machines
- [ ] No unnecessary allocations in hot paths

### Usability
- [ ] API feels natural (feedback from users)
- [ ] Error messages are actionable
- [ ] Documentation examples compile and run
- [ ] Type inference works well

### Production Readiness
- [ ] All public APIs documented
- [ ] Panic-free (uses Result for errors)
- [ ] Semver compliant
- [ ] CI/CD pipeline configured
- [ ] Ready for crates.io publication

---

## Open Questions

1. **Effect Environment Type**
   - Should `StateMachine<E, S>` be generic over environment `E`?
   - Or should transitions be `Effect<E, R>` for any `E`?
   - **Recommendation:** Make environment generic for flexibility

2. **Transition Lookup Performance**
   - Use `Vec` with linear search or `HashMap` for O(1)?
   - **Recommendation:** Start with Vec, optimize if needed

3. **Guard vs Enforcement**
   - Are guards just a special case of enforcement?
   - Should we merge them?
   - **Recommendation:** Keep separate (guards are pre-conditions, enforcement is policy)

4. **Async Support**
   - Should we support async transitions?
   - **Recommendation:** Not in v0.1, revisit if Stillwater adds async Effect

5. **Parallel Transitions**
   - Should multiple transitions be able to execute in parallel?
   - **Recommendation:** Not in v0.1, but design to allow later

---

## Dependencies

```toml
[dependencies]
stillwater = { path = "../stillwater" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }

[dev-dependencies]
proptest = "1.0"
criterion = "0.5"
```

---

## Timeline Estimate

- **Stage 1:** 1-2 weeks (Core foundation)
- **Stage 2:** 1 week (Enforcement)
- **Stage 3:** 1 week (Checkpoint/Resume)
- **Stage 4:** 3-5 days (Builder API)
- **Stage 5:** 1 week (Polish & docs)

**Total:** 4-6 weeks to production-ready 0.1.0

---

## Next Steps

1. Review and refine this plan
2. Setup Cargo project with dependencies
3. Begin Stage 1 implementation
4. Iterate based on real usage

---

*Document Version: 0.1.0*
*Last Updated: 2025-12-01*
