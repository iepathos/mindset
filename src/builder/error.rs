//! Build errors for state machine and transition builders.

use thiserror::Error;

/// Errors that can occur when building state machines and transitions.
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Initial state not specified. Call .initial(state) before .build()")]
    MissingInitialState,

    #[error("No transitions defined. Add at least one transition")]
    NoTransitions,

    #[error("Transition source state not specified. Call .from(state)")]
    MissingFromState,

    #[error("Transition target state not specified. Call .to(state)")]
    MissingToState,

    #[error("Transition action not specified. Call .action(effect) or .succeeds()")]
    MissingAction,
}
