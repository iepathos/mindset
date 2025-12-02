//! State transition types with effectful actions.

use crate::core::{Guard, State};
use std::sync::Arc;
use stillwater::effect::BoxedEffect;

/// Result of executing a transition action.
/// Returned from effectful transition logic.
#[derive(Clone, Debug, PartialEq)]
pub enum TransitionResult<S: State> {
    /// Transition succeeded, move to new state
    Success(S),

    /// Transition should be retried with feedback
    Retry { feedback: String, current_state: S },

    /// Transition failed permanently
    Abort { reason: String, error_state: S },
}

/// Errors that can occur during transitions
#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("No transition available from state '{from}'")]
    NoTransition { from: String },

    #[error("Guard blocked transition from '{from}' to '{to}'")]
    GuardBlocked { from: String, to: String },

    #[error("Transition action failed: {0}")]
    ActionFailed(String),
}

/// Type alias for transition action functions.
/// These functions create fresh effects on each invocation.
pub type TransitionAction<S, Env> =
    Arc<dyn Fn() -> BoxedEffect<TransitionResult<S>, TransitionError, Env> + Send + Sync>;

/// A transition from one state to another with an effectful action.
/// Instead of storing the effect directly, we store a factory function
/// that creates a fresh effect on each execution.
pub struct Transition<S: State, Env> {
    pub from: S,
    pub to: S,
    pub guard: Option<Guard<S>>,
    pub action: TransitionAction<S, Env>,
}

impl<S: State, Env> Transition<S, Env> {
    /// Check if this transition can execute from the current state (pure)
    pub fn can_execute(&self, current: &S) -> bool {
        // Check state match
        if *current != self.from {
            return false;
        }

        // Check guard if present (pure predicate)
        self.guard.as_ref().is_none_or(|g| g.check(current))
    }
}

impl<S: State, Env> Clone for Transition<S, Env> {
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            guard: self.guard.clone(),
            action: Arc::clone(&self.action),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Guard;
    use serde::{Deserialize, Serialize};
    use stillwater::prelude::*;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum TestState {
        Start,
        Middle,
        End,
    }

    impl State for TestState {
        fn name(&self) -> &str {
            match self {
                Self::Start => "Start",
                Self::Middle => "Middle",
                Self::End => "End",
            }
        }

        fn is_final(&self) -> bool {
            matches!(self, Self::End)
        }
    }

    #[test]
    fn can_execute_matches_from_state() {
        let transition: Transition<TestState, ()> = Transition {
            from: TestState::Start,
            to: TestState::Middle,
            guard: None,
            action: Arc::new(|| pure(TransitionResult::Success(TestState::Middle)).boxed()),
        };

        assert!(transition.can_execute(&TestState::Start));
        assert!(!transition.can_execute(&TestState::Middle));
    }

    #[test]
    fn can_execute_respects_guard() {
        let guard = Guard::new(|s: &TestState| s.is_final());

        let transition: Transition<TestState, ()> = Transition {
            from: TestState::End,
            to: TestState::Start,
            guard: Some(guard),
            action: Arc::new(|| pure(TransitionResult::Success(TestState::Start)).boxed()),
        };

        // Should execute - End is final and guard passes
        assert!(transition.can_execute(&TestState::End));

        let transition2: Transition<TestState, ()> = Transition {
            from: TestState::Start,
            to: TestState::Middle,
            guard: Some(Guard::new(|s: &TestState| s.is_final())),
            action: Arc::new(|| pure(TransitionResult::Success(TestState::Middle)).boxed()),
        };

        // Should not execute - Start is not final
        assert!(!transition2.can_execute(&TestState::Start));
    }
}
