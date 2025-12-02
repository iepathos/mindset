//! Mindset: A pure functional state machine library
//!
//! Mindset is built on Stillwater's "pure core, imperative shell" philosophy.
//! The core state machine logic is composed of pure functions with no side effects,
//! while effects are isolated in Effect monads (to be introduced in a future spec).
//!
//! # Core Concepts
//!
//! - **State**: Type-safe state representation via the `State` trait
//! - **Guards**: Pure predicate functions that control transitions
//! - **History**: Immutable tracking of state transitions over time
//!
//! # Example
//!
//! ```rust
//! use mindset::core::{State, StateHistory, StateTransition};
//! use serde::{Deserialize, Serialize};
//! use chrono::Utc;
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
//! let history = StateHistory::new();
//! let transition = StateTransition {
//!     from: WorkflowState::Initial,
//!     to: WorkflowState::Processing,
//!     timestamp: Utc::now(),
//!     attempt: 1,
//! };
//! let history = history.record(transition);
//! ```

pub mod core;

// Re-export commonly used types
pub use core::{Guard, State, StateHistory, StateTransition};
