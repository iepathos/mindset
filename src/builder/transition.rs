//! Builder for constructing state transitions.

use crate::builder::error::BuildError;
use crate::core::{Guard, State};
use crate::effects::{Transition, TransitionError, TransitionResult};
use crate::enforcement::EnforcementRules;
use std::sync::Arc;
use stillwater::effect::BoxedEffect;
use stillwater::prelude::*;

/// Type alias for transition action factories.
type ActionFactory<S, Env> =
    Arc<dyn Fn() -> BoxedEffect<TransitionResult<S>, TransitionError, Env> + Send + Sync>;

/// Builder for constructing transitions with a fluent API.
pub struct TransitionBuilder<S: State, Env> {
    from: Option<S>,
    to: Option<S>,
    guard: Option<Guard<S>>,
    action: Option<ActionFactory<S, Env>>,
    enforcement: Option<EnforcementRules<S>>,
}

impl<S: State + 'static, Env> TransitionBuilder<S, Env> {
    /// Create a new transition builder.
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            guard: None,
            action: None,
            enforcement: None,
        }
    }

    /// Set the source state (required).
    pub fn from(mut self, state: S) -> Self {
        self.from = Some(state);
        self
    }

    /// Set the target state (required).
    pub fn to(mut self, state: S) -> Self {
        self.to = Some(state);
        self
    }

    /// Add a guard predicate (optional).
    pub fn guard(mut self, guard: Guard<S>) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Add a guard using a closure (optional).
    pub fn when<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&S) -> bool + Send + Sync + 'static,
    {
        self.guard = Some(Guard::new(predicate));
        self
    }

    /// Set the action effect (required).
    pub fn action<E>(mut self, effect: E) -> Self
    where
        E: Fn() -> BoxedEffect<TransitionResult<S>, TransitionError, Env> + Send + Sync + 'static,
    {
        self.action = Some(Arc::new(effect));
        self
    }

    /// Set a simple success action.
    /// The target state must be set with `.to()` before calling this.
    pub fn succeeds(self) -> Self
    where
        Env: Clone + Send + Sync + 'static,
    {
        let to = self
            .to
            .clone()
            .expect("to() must be called before succeeds()");
        self.action(move || pure(TransitionResult::Success(to.clone())).boxed())
    }

    /// Add enforcement rules (optional).
    pub fn enforce(mut self, rules: EnforcementRules<S>) -> Self {
        self.enforcement = Some(rules);
        self
    }

    /// Build the transition.
    pub fn build(self) -> Result<Transition<S, Env>, BuildError> {
        let from = self.from.ok_or(BuildError::MissingFromState)?;
        let to = self.to.ok_or(BuildError::MissingToState)?;
        let action = self.action.ok_or(BuildError::MissingAction)?;

        Ok(Transition {
            from,
            to,
            guard: self.guard,
            action,
            enforcement: self.enforcement,
        })
    }
}

impl<S: State + 'static, Env> Default for TransitionBuilder<S, Env> {
    fn default() -> Self {
        Self::new()
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
    }

    #[test]
    fn builder_validates_required_fields() {
        let result = TransitionBuilder::<TestState, ()>::new()
            .from(TestState::Initial)
            .build();

        assert!(matches!(result, Err(BuildError::MissingToState)));
    }

    #[test]
    fn builder_validates_missing_action() {
        let result = TransitionBuilder::<TestState, ()>::new()
            .from(TestState::Initial)
            .to(TestState::Processing)
            .build();

        assert!(matches!(result, Err(BuildError::MissingAction)));
    }

    #[test]
    fn succeeds_requires_to_state() {
        let result = std::panic::catch_unwind(|| {
            TransitionBuilder::<TestState, ()>::new()
                .from(TestState::Initial)
                .succeeds()
        });

        assert!(result.is_err());
    }

    #[test]
    fn transition_builder_with_guard() {
        let transition: Transition<TestState, ()> = TransitionBuilder::new()
            .from(TestState::Initial)
            .to(TestState::Processing)
            .when(|s: &TestState| !s.is_final())
            .succeeds()
            .build()
            .unwrap();

        assert!(transition.can_execute(&TestState::Initial));
        assert!(!transition.can_execute(&TestState::Complete));
    }

    #[test]
    fn fluent_api_builds_transition() {
        let transition: Result<Transition<TestState, ()>, _> = TransitionBuilder::new()
            .from(TestState::Initial)
            .to(TestState::Processing)
            .succeeds()
            .build();

        assert!(transition.is_ok());
        let transition = transition.unwrap();
        assert_eq!(transition.from, TestState::Initial);
        assert_eq!(transition.to, TestState::Processing);
    }
}
