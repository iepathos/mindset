//! Builder API for ergonomic state machine construction.
//!
//! This module provides fluent builders and macros for creating state machines
//! with minimal boilerplate while maintaining type safety.

pub mod error;
pub mod machine;
pub mod macros;
pub mod transition;

pub use error::BuildError;
pub use machine::StateMachineBuilder;
pub use transition::TransitionBuilder;

use crate::core::State;
use crate::effects::{Transition, TransitionResult};
use stillwater::prelude::*;

/// Create a simple unconditional transition that succeeds.
///
/// # Example
///
/// ```
/// use mindset::builder::simple_transition;
/// use mindset::state_enum;
///
/// state_enum! {
///     enum MyState {
///         Start,
///         End,
///     }
///     final: [End]
/// }
///
/// let transition = simple_transition::<MyState, ()>(MyState::Start, MyState::End);
/// ```
pub fn simple_transition<S, Env>(from: S, to: S) -> Transition<S, Env>
where
    S: State + 'static,
    Env: Clone + Send + Sync + 'static,
{
    let to_clone = to.clone();
    TransitionBuilder::new()
        .from(from)
        .to(to)
        .action(move || pure(TransitionResult::Success(to_clone.clone())).boxed())
        .build()
        .expect("Simple transition should always build")
}

/// Create a transition with a guard predicate.
///
/// # Example
///
/// ```
/// use mindset::builder::guarded_transition;
/// use mindset::state_enum;
/// use mindset::core::State;
///
/// state_enum! {
///     enum MyState {
///         Start,
///         Middle,
///         End,
///     }
///     final: [End]
/// }
///
/// let transition = guarded_transition::<MyState, (), _>(
///     MyState::Start,
///     MyState::Middle,
///     |s| !s.is_final()
/// );
/// ```
pub fn guarded_transition<S, Env, F>(from: S, to: S, guard: F) -> Transition<S, Env>
where
    S: State + 'static,
    Env: Clone + Send + Sync + 'static,
    F: Fn(&S) -> bool + Send + Sync + 'static,
{
    let to_clone = to.clone();
    TransitionBuilder::new()
        .from(from)
        .to(to)
        .when(guard)
        .action(move || pure(TransitionResult::Success(to_clone.clone())).boxed())
        .build()
        .expect("Guarded transition should always build")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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
    fn simple_transition_builds() {
        let transition = simple_transition::<TestState, ()>(TestState::Start, TestState::Middle);

        assert_eq!(transition.from, TestState::Start);
        assert_eq!(transition.to, TestState::Middle);
        assert!(transition.can_execute(&TestState::Start));
    }

    #[test]
    fn guarded_transition_respects_guard() {
        let transition =
            guarded_transition::<TestState, (), _>(TestState::Start, TestState::Middle, |s| {
                !s.is_final()
            });

        assert!(transition.can_execute(&TestState::Start));
        assert!(!transition.can_execute(&TestState::End));
    }
}
