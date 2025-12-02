//! Guard predicates for controlling state transitions.
//!
//! Guards are pure boolean functions that determine whether a transition
//! can execute. They enable declarative transition rules without side effects.

use super::state::State;
use std::marker::PhantomData;

/// Pure predicate that determines if a transition can execute.
///
/// Guards are evaluated before attempting a transition. They encapsulate
/// pre-conditions as pure functions, maintaining the "pure core" philosophy.
///
/// # Example
///
/// ```rust
/// use mindset::core::{Guard, State};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// enum TaskState {
///     Pending,
///     Running,
///     Complete,
/// }
///
/// impl State for TaskState {
///     fn name(&self) -> &str {
///         match self {
///             Self::Pending => "Pending",
///             Self::Running => "Running",
///             Self::Complete => "Complete",
///         }
///     }
///
///     fn is_final(&self) -> bool {
///         matches!(self, Self::Complete)
///     }
/// }
///
/// // Guard that only allows transitions from non-final states
/// let can_transition = Guard::new(|state: &TaskState| !state.is_final());
///
/// assert!(can_transition.check(&TaskState::Pending));
/// assert!(can_transition.check(&TaskState::Running));
/// assert!(!can_transition.check(&TaskState::Complete));
/// ```
pub struct Guard<S: State> {
    predicate: Box<dyn Fn(&S) -> bool + Send + Sync>,
    _phantom: PhantomData<S>,
}

impl<S: State> Guard<S> {
    /// Create a guard from a pure predicate function.
    ///
    /// The predicate must be pure (deterministic, no side effects) and
    /// thread-safe (Send + Sync).
    ///
    /// # Example
    ///
    /// ```rust
    /// use mindset::core::{Guard, State};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    /// enum WorkState {
    ///     Idle,
    ///     Busy,
    /// }
    ///
    /// impl State for WorkState {
    ///     fn name(&self) -> &str {
    ///         match self {
    ///             Self::Idle => "Idle",
    ///             Self::Busy => "Busy",
    ///         }
    ///     }
    /// }
    ///
    /// let only_from_idle = Guard::new(|s: &WorkState| matches!(s, WorkState::Idle));
    /// ```
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn(&S) -> bool + Send + Sync + 'static,
    {
        Guard {
            predicate: Box::new(predicate),
            _phantom: PhantomData,
        }
    }

    /// Check if the guard allows transition from this state.
    ///
    /// This is a pure function that evaluates the predicate without
    /// any side effects.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mindset::core::{Guard, State};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    /// enum Status {
    ///     Active,
    ///     Inactive,
    /// }
    ///
    /// impl State for Status {
    ///     fn name(&self) -> &str {
    ///         match self {
    ///             Self::Active => "Active",
    ///             Self::Inactive => "Inactive",
    ///         }
    ///     }
    /// }
    ///
    /// let guard = Guard::new(|s: &Status| matches!(s, Status::Active));
    ///
    /// assert!(guard.check(&Status::Active));
    /// assert!(!guard.check(&Status::Inactive));
    /// ```
    pub fn check(&self, state: &S) -> bool {
        (self.predicate)(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum TestState {
        Initial,
        Processing,
        Complete,
        Failed,
    }

    impl State for TestState {
        fn name(&self) -> &str {
            match self {
                Self::Initial => "Initial",
                Self::Processing => "Processing",
                Self::Complete => "Complete",
                Self::Failed => "Failed",
            }
        }

        fn is_final(&self) -> bool {
            matches!(self, Self::Complete | Self::Failed)
        }

        fn is_error(&self) -> bool {
            matches!(self, Self::Failed)
        }
    }

    #[test]
    fn guard_allows_matching_states() {
        let guard = Guard::new(|s: &TestState| matches!(s, TestState::Initial));

        assert!(guard.check(&TestState::Initial));
        assert!(!guard.check(&TestState::Processing));
    }

    #[test]
    fn guard_checks_non_final_states() {
        let guard = Guard::new(|s: &TestState| !s.is_final());

        assert!(guard.check(&TestState::Initial));
        assert!(guard.check(&TestState::Processing));
        assert!(!guard.check(&TestState::Complete));
        assert!(!guard.check(&TestState::Failed));
    }

    #[test]
    fn guard_checks_non_error_states() {
        let guard = Guard::new(|s: &TestState| !s.is_error());

        assert!(guard.check(&TestState::Initial));
        assert!(guard.check(&TestState::Processing));
        assert!(guard.check(&TestState::Complete));
        assert!(!guard.check(&TestState::Failed));
    }

    #[test]
    fn guard_is_deterministic() {
        let state = TestState::Processing;
        let guard = Guard::new(|s: &TestState| !s.is_final());

        let result1 = guard.check(&state);
        let result2 = guard.check(&state);

        assert_eq!(result1, result2);
    }

    #[test]
    fn guard_can_use_complex_predicates() {
        let guard =
            Guard::new(|s: &TestState| matches!(s, TestState::Initial | TestState::Processing));

        assert!(guard.check(&TestState::Initial));
        assert!(guard.check(&TestState::Processing));
        assert!(!guard.check(&TestState::Complete));
        assert!(!guard.check(&TestState::Failed));
    }
}
