//! State transition history tracking.
//!
//! Provides immutable tracking of state machine transitions over time,
//! following functional programming principles.

use super::state::State;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Record of a single state transition.
///
/// Transitions are immutable values representing a move from one state
/// to another at a specific point in time.
///
/// # Example
///
/// ```rust
/// use mindset::core::{State, StateTransition};
/// use serde::{Deserialize, Serialize};
/// use chrono::Utc;
///
/// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// enum TaskState {
///     Pending,
///     Running,
/// }
///
/// impl State for TaskState {
///     fn name(&self) -> &str {
///         match self {
///             Self::Pending => "Pending",
///             Self::Running => "Running",
///         }
///     }
/// }
///
/// let transition = StateTransition {
///     from: TaskState::Pending,
///     to: TaskState::Running,
///     timestamp: Utc::now(),
///     attempt: 1,
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct StateTransition<S: State> {
    /// The state being transitioned from
    pub from: S,
    /// The state being transitioned to
    pub to: S,
    /// When the transition occurred
    pub timestamp: DateTime<Utc>,
    /// The attempt number for this transition (for retry logic)
    pub attempt: usize,
}

/// Ordered history of state transitions.
///
/// History is immutable - the `record` method returns a new history
/// with the transition added, following functional programming principles.
///
/// # Example
///
/// ```rust
/// use mindset::core::{State, StateHistory, StateTransition};
/// use serde::{Deserialize, Serialize};
/// use chrono::Utc;
///
/// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// enum WorkState {
///     Start,
///     Middle,
///     End,
/// }
///
/// impl State for WorkState {
///     fn name(&self) -> &str {
///         match self {
///             Self::Start => "Start",
///             Self::Middle => "Middle",
///             Self::End => "End",
///         }
///     }
/// }
///
/// let history = StateHistory::new();
///
/// let transition1 = StateTransition {
///     from: WorkState::Start,
///     to: WorkState::Middle,
///     timestamp: Utc::now(),
///     attempt: 1,
/// };
///
/// let history = history.record(transition1);
///
/// let transition2 = StateTransition {
///     from: WorkState::Middle,
///     to: WorkState::End,
///     timestamp: Utc::now(),
///     attempt: 1,
/// };
///
/// let history = history.record(transition2);
///
/// let path = history.get_path();
/// assert_eq!(path.len(), 3); // Start -> Middle -> End
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct StateHistory<S: State> {
    transitions: Vec<StateTransition<S>>,
}

impl<S: State> Default for StateHistory<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: State> StateHistory<S> {
    /// Create a new empty history.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mindset::core::{State, StateHistory};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    /// enum Status { Active }
    ///
    /// impl State for Status {
    ///     fn name(&self) -> &str { "Active" }
    /// }
    ///
    /// let history: StateHistory<Status> = StateHistory::new();
    /// assert_eq!(history.transitions().len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }

    /// Record a transition, returning a new history.
    ///
    /// This is a pure function - it does not mutate the existing history
    /// but returns a new one with the transition added.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mindset::core::{State, StateHistory, StateTransition};
    /// use serde::{Deserialize, Serialize};
    /// use chrono::Utc;
    ///
    /// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    /// enum Step { A, B }
    ///
    /// impl State for Step {
    ///     fn name(&self) -> &str {
    ///         match self {
    ///             Self::A => "A",
    ///             Self::B => "B",
    ///         }
    ///     }
    /// }
    ///
    /// let history = StateHistory::new();
    /// let transition = StateTransition {
    ///     from: Step::A,
    ///     to: Step::B,
    ///     timestamp: Utc::now(),
    ///     attempt: 1,
    /// };
    ///
    /// let new_history = history.record(transition);
    /// assert_eq!(new_history.transitions().len(), 1);
    /// assert_eq!(history.transitions().len(), 0); // Original unchanged
    /// ```
    pub fn record(&self, transition: StateTransition<S>) -> Self {
        let mut transitions = self.transitions.clone();
        transitions.push(transition);
        Self { transitions }
    }

    /// Get the path of states traversed.
    ///
    /// Returns references to states in order: initial state, then
    /// the `to` state of each transition.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mindset::core::{State, StateHistory, StateTransition};
    /// use serde::{Deserialize, Serialize};
    /// use chrono::Utc;
    ///
    /// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    /// enum Phase { One, Two, Three }
    ///
    /// impl State for Phase {
    ///     fn name(&self) -> &str {
    ///         match self {
    ///             Self::One => "One",
    ///             Self::Two => "Two",
    ///             Self::Three => "Three",
    ///         }
    ///     }
    /// }
    ///
    /// let mut history = StateHistory::new();
    ///
    /// history = history.record(StateTransition {
    ///     from: Phase::One,
    ///     to: Phase::Two,
    ///     timestamp: Utc::now(),
    ///     attempt: 1,
    /// });
    ///
    /// history = history.record(StateTransition {
    ///     from: Phase::Two,
    ///     to: Phase::Three,
    ///     timestamp: Utc::now(),
    ///     attempt: 1,
    /// });
    ///
    /// let path = history.get_path();
    /// assert_eq!(path.len(), 3);
    /// assert_eq!(path[0], &Phase::One);
    /// assert_eq!(path[1], &Phase::Two);
    /// assert_eq!(path[2], &Phase::Three);
    /// ```
    pub fn get_path(&self) -> Vec<&S> {
        let mut path = Vec::new();
        if let Some(first) = self.transitions.first() {
            path.push(&first.from);
        }
        for transition in &self.transitions {
            path.push(&transition.to);
        }
        path
    }

    /// Calculate total duration from first to last transition.
    ///
    /// Returns `None` if there are no transitions. Otherwise returns
    /// the duration between the first and last transition timestamps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mindset::core::{State, StateHistory, StateTransition};
    /// use serde::{Deserialize, Serialize};
    /// use chrono::Utc;
    ///
    /// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    /// enum State1 { A, B }
    ///
    /// impl State for State1 {
    ///     fn name(&self) -> &str {
    ///         match self {
    ///             Self::A => "A",
    ///             Self::B => "B",
    ///         }
    ///     }
    /// }
    ///
    /// let history = StateHistory::new();
    /// assert!(history.duration().is_none());
    ///
    /// let start = Utc::now();
    /// let history = history.record(StateTransition {
    ///     from: State1::A,
    ///     to: State1::B,
    ///     timestamp: start,
    ///     attempt: 1,
    /// });
    ///
    /// assert!(history.duration().is_some());
    /// ```
    pub fn duration(&self) -> Option<Duration> {
        if let (Some(first), Some(last)) = (self.transitions.first(), self.transitions.last()) {
            let duration = last.timestamp.signed_duration_since(first.timestamp);
            duration.to_std().ok()
        } else {
            None
        }
    }

    /// Get all transitions.
    ///
    /// Returns a slice of all recorded transitions in order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mindset::core::{State, StateHistory, StateTransition};
    /// use serde::{Deserialize, Serialize};
    /// use chrono::Utc;
    ///
    /// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    /// enum MyState { X, Y }
    ///
    /// impl State for MyState {
    ///     fn name(&self) -> &str {
    ///         match self {
    ///             Self::X => "X",
    ///             Self::Y => "Y",
    ///         }
    ///     }
    /// }
    ///
    /// let history = StateHistory::new();
    /// let history = history.record(StateTransition {
    ///     from: MyState::X,
    ///     to: MyState::Y,
    ///     timestamp: Utc::now(),
    ///     attempt: 1,
    /// });
    ///
    /// assert_eq!(history.transitions().len(), 1);
    /// ```
    pub fn transitions(&self) -> &[StateTransition<S>] {
        &self.transitions
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
    fn new_history_is_empty() {
        let history: StateHistory<TestState> = StateHistory::new();
        assert_eq!(history.transitions().len(), 0);
        assert!(history.get_path().is_empty());
        assert!(history.duration().is_none());
    }

    #[test]
    fn record_adds_transition() {
        let history = StateHistory::new();

        let transition = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: Utc::now(),
            attempt: 1,
        };

        let history = history.record(transition);

        assert_eq!(history.transitions().len(), 1);
    }

    #[test]
    fn record_is_immutable() {
        let history = StateHistory::new();

        let transition = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: Utc::now(),
            attempt: 1,
        };

        let new_history = history.record(transition);

        assert_eq!(history.transitions().len(), 0);
        assert_eq!(new_history.transitions().len(), 1);
    }

    #[test]
    fn get_path_returns_state_sequence() {
        let mut history = StateHistory::new();

        let transition1 = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: Utc::now(),
            attempt: 1,
        };

        history = history.record(transition1);

        let transition2 = StateTransition {
            from: TestState::Processing,
            to: TestState::Complete,
            timestamp: Utc::now(),
            attempt: 1,
        };

        history = history.record(transition2);

        let path = history.get_path();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], &TestState::Initial);
        assert_eq!(path[1], &TestState::Processing);
        assert_eq!(path[2], &TestState::Complete);
    }

    #[test]
    fn duration_calculates_elapsed_time() {
        let history = StateHistory::new();
        let start = Utc::now();

        let transition1 = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: start,
            attempt: 1,
        };

        let history = history.record(transition1);

        std::thread::sleep(std::time::Duration::from_millis(10));

        let transition2 = StateTransition {
            from: TestState::Processing,
            to: TestState::Complete,
            timestamp: Utc::now(),
            attempt: 1,
        };

        let history = history.record(transition2);

        let duration = history.duration();
        assert!(duration.is_some());
        assert!(duration.unwrap() >= std::time::Duration::from_millis(10));
    }

    #[test]
    fn history_serializes_correctly() {
        let mut history = StateHistory::new();

        let transition = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: Utc::now(),
            attempt: 1,
        };

        history = history.record(transition);

        let json = serde_json::to_string(&history).unwrap();
        let deserialized: StateHistory<TestState> = serde_json::from_str(&json).unwrap();

        assert_eq!(
            history.transitions().len(),
            deserialized.transitions().len()
        );
    }

    #[test]
    fn single_transition_has_duration_zero() {
        let timestamp = Utc::now();

        let transition = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp,
            attempt: 1,
        };

        let history = StateHistory::new().record(transition);

        let duration = history.duration();
        assert!(duration.is_some());
        assert_eq!(duration.unwrap(), std::time::Duration::from_secs(0));
    }

    #[test]
    fn attempt_field_is_tracked() {
        let transition = StateTransition {
            from: TestState::Initial,
            to: TestState::Processing,
            timestamp: Utc::now(),
            attempt: 3,
        };

        assert_eq!(transition.attempt, 3);
    }
}
