//! Builder for constructing state machines.

use crate::builder::error::BuildError;
use crate::builder::transition::TransitionBuilder;
use crate::core::State;
use crate::effects::{StateMachine, Transition};
use std::marker::PhantomData;

/// Builder for constructing state machines with a fluent API.
pub struct StateMachineBuilder<S: State + 'static, Env: Clone + Send + Sync + 'static> {
    initial: Option<S>,
    transitions: Vec<Transition<S, Env>>,
    _phantom: PhantomData<Env>,
}

impl<S: State + 'static, Env: Clone + Send + Sync + 'static> StateMachineBuilder<S, Env> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            initial: None,
            transitions: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Set the initial state (required).
    pub fn initial(mut self, state: S) -> Self {
        self.initial = Some(state);
        self
    }

    /// Add a transition using a builder.
    /// Returns an error if the builder fails validation.
    pub fn transition(mut self, builder: TransitionBuilder<S, Env>) -> Result<Self, BuildError> {
        let transition = builder.build()?;
        self.transitions.push(transition);
        Ok(self)
    }

    /// Add a pre-built transition.
    pub fn add_transition(mut self, transition: Transition<S, Env>) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Add multiple transitions at once.
    pub fn transitions(mut self, transitions: Vec<Transition<S, Env>>) -> Self {
        self.transitions.extend(transitions);
        self
    }

    /// Build the state machine.
    /// Returns an error if required fields are missing.
    pub fn build(self) -> Result<StateMachine<S, Env>, BuildError> {
        let initial = self.initial.ok_or(BuildError::MissingInitialState)?;

        if self.transitions.is_empty() {
            return Err(BuildError::NoTransitions);
        }

        let mut machine = StateMachine::new(initial);
        for transition in self.transitions {
            machine.add_transition(transition);
        }

        Ok(machine)
    }
}

impl<S: State + 'static, Env: Clone + Send + Sync + 'static> Default
    for StateMachineBuilder<S, Env>
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::TransitionResult;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use stillwater::prelude::*;

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
    }

    #[test]
    fn builder_validates_required_fields() {
        let result = StateMachineBuilder::<TestState, ()>::new().build();

        assert!(matches!(result, Err(BuildError::MissingInitialState)));
    }

    #[test]
    fn builder_requires_transitions() {
        let result = StateMachineBuilder::<TestState, ()>::new()
            .initial(TestState::Initial)
            .build();

        assert!(matches!(result, Err(BuildError::NoTransitions)));
    }

    #[test]
    fn fluent_api_builds_machine() {
        let transition1: Transition<TestState, ()> = Transition {
            from: TestState::Initial,
            to: TestState::Processing,
            guard: None,
            action: Arc::new(|| pure(TransitionResult::Success(TestState::Processing)).boxed()),
            enforcement: None,
        };

        let transition2: Transition<TestState, ()> = Transition {
            from: TestState::Processing,
            to: TestState::Complete,
            guard: None,
            action: Arc::new(|| pure(TransitionResult::Success(TestState::Complete)).boxed()),
            enforcement: None,
        };

        let machine = StateMachineBuilder::new()
            .initial(TestState::Initial)
            .add_transition(transition1)
            .add_transition(transition2)
            .build();

        assert!(machine.is_ok());
        let machine = machine.unwrap();
        assert_eq!(machine.current_state(), &TestState::Initial);
    }

    #[test]
    fn add_multiple_transitions() {
        let transitions: Vec<Transition<TestState, ()>> = vec![
            Transition {
                from: TestState::Initial,
                to: TestState::Processing,
                guard: None,
                action: Arc::new(|| pure(TransitionResult::Success(TestState::Processing)).boxed()),
                enforcement: None,
            },
            Transition {
                from: TestState::Processing,
                to: TestState::Complete,
                guard: None,
                action: Arc::new(|| pure(TransitionResult::Success(TestState::Complete)).boxed()),
                enforcement: None,
            },
        ];

        let machine = StateMachineBuilder::new()
            .initial(TestState::Initial)
            .transitions(transitions)
            .build();

        assert!(machine.is_ok());
    }
}
