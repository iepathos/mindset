//! Core state machine types and logic.
//!
//! This module contains the pure functional core of the state machine:
//! - State definitions via the `State` trait
//! - Guard predicates for transition control
//! - Immutable history tracking
//!
//! All logic in this module is pure (no side effects), following
//! the "pure core, imperative shell" philosophy.

mod guard;
mod history;
mod state;

pub use guard::Guard;
pub use history::{StateHistory, StateTransition};
pub use state::State;
