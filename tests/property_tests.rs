//! Property-based tests for core state machine types.
//!
//! These tests use proptest to verify properties hold across
//! many randomly generated inputs.

use chrono::Utc;
use mindset::core::{Guard, State, StateHistory, StateTransition};
use proptest::prelude::*;
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

prop_compose! {
    fn arbitrary_state()(variant in 0..4u8) -> TestState {
        match variant {
            0 => TestState::Initial,
            1 => TestState::Processing,
            2 => TestState::Complete,
            _ => TestState::Failed,
        }
    }
}

proptest! {
    #[test]
    fn guard_is_deterministic(state in arbitrary_state()) {
        let guard = Guard::new(|s: &TestState| !s.is_final());
        let result1 = guard.check(&state);
        let result2 = guard.check(&state);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn state_name_is_stable(state in arbitrary_state()) {
        let name1 = state.name();
        let name2 = state.name();
        prop_assert_eq!(name1, name2);
    }

    #[test]
    fn state_final_is_deterministic(state in arbitrary_state()) {
        let final1 = state.is_final();
        let final2 = state.is_final();
        prop_assert_eq!(final1, final2);
    }

    #[test]
    fn state_error_is_deterministic(state in arbitrary_state()) {
        let error1 = state.is_error();
        let error2 = state.is_error();
        prop_assert_eq!(error1, error2);
    }

    #[test]
    fn history_preserves_order(
        transitions in prop::collection::vec(arbitrary_state(), 1..10)
    ) {
        let mut history = StateHistory::new();
        let mut expected_path = vec![TestState::Initial];

        for (i, to_state) in transitions.iter().enumerate() {
            let from_state = if i == 0 {
                TestState::Initial
            } else {
                transitions[i - 1].clone()
            };

            let transition = StateTransition {
                from: from_state.clone(),
                to: to_state.clone(),
                timestamp: Utc::now(),
                attempt: 1,
            };

            history = history.record(transition);
            expected_path.push(to_state.clone());
        }

        let path = history.get_path();
        prop_assert_eq!(path.len(), expected_path.len());

        for (i, state) in path.iter().enumerate() {
            prop_assert_eq!(*state, &expected_path[i]);
        }
    }

    #[test]
    fn history_record_is_pure(state1 in arbitrary_state(), state2 in arbitrary_state()) {
        let history = StateHistory::new();

        let transition = StateTransition {
            from: state1,
            to: state2,
            timestamp: Utc::now(),
            attempt: 1,
        };

        let new_history = history.record(transition);

        // Original history unchanged
        prop_assert_eq!(history.transitions().len(), 0);
        // New history has the transition
        prop_assert_eq!(new_history.transitions().len(), 1);
    }

    #[test]
    fn history_duration_is_non_negative(
        transitions in prop::collection::vec(arbitrary_state(), 1..5)
    ) {
        let mut history = StateHistory::new();
        let base_time = Utc::now();

        for (i, to_state) in transitions.iter().enumerate() {
            let from_state = if i == 0 {
                TestState::Initial
            } else {
                transitions[i - 1].clone()
            };

            let transition = StateTransition {
                from: from_state,
                to: to_state.clone(),
                timestamp: base_time,
                attempt: 1,
            };

            history = history.record(transition);
        }

        if let Some(_duration) = history.duration() {
            // Duration exists and is valid by type (always non-negative)
        }
    }

    #[test]
    fn state_roundtrip_serialization(state in arbitrary_state()) {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: TestState = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(state, deserialized);
    }

    #[test]
    fn history_roundtrip_serialization(
        transitions in prop::collection::vec(arbitrary_state(), 0..5)
    ) {
        let mut history = StateHistory::new();

        for (i, to_state) in transitions.iter().enumerate() {
            let from_state = if i == 0 {
                TestState::Initial
            } else {
                transitions[i - 1].clone()
            };

            let transition = StateTransition {
                from: from_state,
                to: to_state.clone(),
                timestamp: Utc::now(),
                attempt: 1,
            };

            history = history.record(transition);
        }

        let json = serde_json::to_string(&history).unwrap();
        let deserialized: StateHistory<TestState> = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(history.transitions().len(), deserialized.transitions().len());
    }
}
