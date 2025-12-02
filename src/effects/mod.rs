//! Effectful state machine operations using Stillwater 0.11.0.
//!
//! This module provides the "imperative shell" around the pure core,
//! implementing state transitions with side effects, I/O, and external calls.
//!
//! # Key Concepts
//!
//! - **Transitions**: Define state changes with guards and effectful actions
//! - **State Machine**: Executes transitions and tracks state history
//! - **Effects**: Uses Stillwater's zero-cost effect system
//!
//! # Zero-Cost Abstractions
//!
//! Following Stillwater 0.11.0 conventions:
//! - Functions return `impl Effect` for zero-cost composition
//! - Collections store `BoxedEffect` (one allocation per transition)
//! - Use free-standing constructors: `pure()`, `fail()`, `from_fn()`

mod machine;
mod transition;

pub use machine::{StateMachine, StepResult};
pub use transition::{Transition, TransitionError, TransitionResult};
