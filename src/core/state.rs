//! Core State trait for state machine states.
//!
//! All state machine states must implement this trait, which provides
//! pure methods for inspecting state properties without side effects.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait for state machine states.
///
/// All methods are pure - no side effects. States represent immutable
/// values that describe the current position in a state machine.
///
/// # Required Traits
///
/// - `Clone`: States must be cloneable for history tracking
/// - `PartialEq`: States must be comparable for transition logic
/// - `Debug`: States must be debuggable for diagnostics
/// - `Serialize` + `Deserialize`: States must be serializable for persistence
///
/// # Example
///
/// ```rust
/// use mindset::core::State;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// enum TaskState {
///     Pending,
///     Running,
///     Complete,
///     Failed,
/// }
///
/// impl State for TaskState {
///     fn name(&self) -> &str {
///         match self {
///             Self::Pending => "Pending",
///             Self::Running => "Running",
///             Self::Complete => "Complete",
///             Self::Failed => "Failed",
///         }
///     }
///
///     fn is_final(&self) -> bool {
///         matches!(self, Self::Complete | Self::Failed)
///     }
///
///     fn is_error(&self) -> bool {
///         matches!(self, Self::Failed)
///     }
/// }
/// ```
pub trait State:
    Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync
{
    /// Get the state's name for display/logging.
    ///
    /// Returns a static string reference for zero-cost naming.
    fn name(&self) -> &str;

    /// Check if this is a final (terminal) state.
    ///
    /// Final states represent completion points in the state machine
    /// where no further transitions are expected.
    ///
    /// Default implementation returns `false`.
    fn is_final(&self) -> bool {
        false
    }

    /// Check if this is an error state.
    ///
    /// Error states represent failure conditions in the state machine.
    /// They are typically also final states, but this is not enforced.
    ///
    /// Default implementation returns `false`.
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn state_name_returns_correct_value() {
        assert_eq!(TestState::Initial.name(), "Initial");
        assert_eq!(TestState::Processing.name(), "Processing");
        assert_eq!(TestState::Complete.name(), "Complete");
        assert_eq!(TestState::Failed.name(), "Failed");
    }

    #[test]
    fn is_final_identifies_terminal_states() {
        assert!(!TestState::Initial.is_final());
        assert!(!TestState::Processing.is_final());
        assert!(TestState::Complete.is_final());
        assert!(TestState::Failed.is_final());
    }

    #[test]
    fn is_error_identifies_error_states() {
        assert!(!TestState::Initial.is_error());
        assert!(!TestState::Processing.is_error());
        assert!(!TestState::Complete.is_error());
        assert!(TestState::Failed.is_error());
    }

    #[test]
    fn state_trait_methods_work() {
        let state = TestState::Initial;
        assert_eq!(state.name(), "Initial");
        assert!(!state.is_final());
        assert!(!state.is_error());

        let complete = TestState::Complete;
        assert!(complete.is_final());
        assert!(!complete.is_error());

        let failed = TestState::Failed;
        assert!(failed.is_final());
        assert!(failed.is_error());
    }

    #[test]
    fn state_serializes_correctly() {
        let state = TestState::Initial;
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: TestState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn state_is_cloneable() {
        let state = TestState::Processing;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn state_is_comparable() {
        let state1 = TestState::Processing;
        let state2 = TestState::Processing;
        let state3 = TestState::Complete;

        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }
}
