//! Mindset: A pure functional state machine library
//!
//! Mindset is built on Stillwater's "pure core, imperative shell" philosophy.
//! The core state machine logic is composed of pure functions with no side effects,
//! while effects are isolated in Effect monads using Stillwater 0.11.0.
//!
//! # Core Concepts
//!
//! - **State**: Type-safe state representation via the `State` trait
//! - **Guards**: Pure predicate functions that control transitions
//! - **History**: Immutable tracking of state transitions over time
//! - **Effects**: Effectful state transitions using Stillwater's zero-cost effect system
//!
//! # Example
//!
//! ```rust
//! use mindset::core::{State, StateHistory, StateTransition};
//! use mindset::effects::{StateMachine, Transition, TransitionResult};
//! use serde::{Deserialize, Serialize};
//! use chrono::Utc;
//! use stillwater::prelude::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
//! enum WorkflowState {
//!     Initial,
//!     Processing,
//!     Complete,
//! }
//!
//! impl State for WorkflowState {
//!     fn name(&self) -> &str {
//!         match self {
//!             Self::Initial => "Initial",
//!             Self::Processing => "Processing",
//!             Self::Complete => "Complete",
//!         }
//!     }
//!
//!     fn is_final(&self) -> bool {
//!         matches!(self, Self::Complete)
//!     }
//! }
//!
//! // Create a state machine with effectful transitions
//! let mut machine: StateMachine<WorkflowState, ()> = StateMachine::new(WorkflowState::Initial);
//!
//! // Add a transition with an action factory
//! machine.add_transition(Transition {
//!     from: WorkflowState::Initial,
//!     to: WorkflowState::Processing,
//!     guard: None,
//!     action: Arc::new(|| pure(TransitionResult::Success(WorkflowState::Processing)).boxed()),
//!     enforcement: None,
//! });
//! ```

pub mod checkpoint;
pub mod core;
pub mod effects;
pub mod enforcement;

// Re-export commonly used types
pub use checkpoint::{Checkpoint, CheckpointError, MachineMetadata, CHECKPOINT_VERSION};
pub use core::{Guard, State, StateHistory, StateTransition};
pub use effects::{StateMachine, StepResult, Transition, TransitionError, TransitionResult};
pub use enforcement::{
    EnforcementBuilder, EnforcementRules, TransitionContext, ViolationError, ViolationStrategy,
};
