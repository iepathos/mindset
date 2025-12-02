//! State machine that executes effectful transitions.

use crate::core::{State, StateHistory, StateTransition};
use crate::effects::transition::{Transition, TransitionError, TransitionResult};
use chrono::Utc;
use stillwater::effect::Effect;
use stillwater::prelude::*;

/// Result of executing a single step
#[derive(Clone, Debug, PartialEq)]
pub enum StepResult<S: State> {
    /// Successfully transitioned to new state
    Transitioned(S),

    /// Transition should be retried
    Retry { feedback: String, attempts: usize },

    /// Transition aborted permanently
    Aborted { reason: String, error_state: S },
}

/// State machine that executes effectful transitions.
pub struct StateMachine<S: State + 'static, Env: Clone + Send + Sync + 'static> {
    current: S,
    transitions: Vec<Transition<S, Env>>,
    history: StateHistory<S>,
    attempt_count: usize,
}

impl<S: State + 'static, Env: Clone + Send + Sync + 'static> StateMachine<S, Env> {
    /// Create a new state machine in the initial state
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            transitions: Vec::new(),
            history: StateHistory::new(),
            attempt_count: 0,
        }
    }

    /// Add a transition to the machine
    pub fn add_transition(&mut self, transition: Transition<S, Env>) {
        self.transitions.push(transition);
    }

    /// Get current state (pure)
    pub fn current_state(&self) -> &S {
        &self.current
    }

    /// Check if machine is in a final state (pure)
    pub fn is_final(&self) -> bool {
        self.current.is_final()
    }

    /// Get state history (pure)
    pub fn history(&self) -> &StateHistory<S> {
        &self.history
    }

    /// Execute one step of the state machine.
    /// Returns impl Effect for zero-cost composition.
    /// After running the effect, call apply_result() to update the machine state.
    pub fn step(
        &self,
    ) -> impl Effect<Output = (S, StepResult<S>, usize), Error = TransitionError, Env = Env> + '_
    {
        // Find applicable transition (pure)
        let transition_opt = self
            .transitions
            .iter()
            .find(|t| t.can_execute(&self.current));

        let Some(transition) = transition_opt else {
            return fail(TransitionError::NoTransition {
                from: self.current.name().to_string(),
            })
            .boxed();
        };

        // Get fresh effect from action factory
        let from_state = self.current.clone();
        let attempt_count = self.attempt_count;
        let action = (transition.action)();

        // Execute action and return result with context
        action
            .map(move |result| {
                let step_result = match &result {
                    TransitionResult::Success(new_state) => {
                        StepResult::Transitioned(new_state.clone())
                    }
                    TransitionResult::Retry {
                        feedback,
                        current_state: _,
                    } => StepResult::Retry {
                        feedback: feedback.clone(),
                        attempts: attempt_count + 1,
                    },
                    TransitionResult::Abort {
                        reason,
                        error_state,
                    } => StepResult::Aborted {
                        reason: reason.clone(),
                        error_state: error_state.clone(),
                    },
                };
                (from_state.clone(), step_result, attempt_count)
            })
            .boxed()
    }

    /// Apply the result from step() to update machine state.
    /// Call this after running the effect.
    pub fn apply_result(&mut self, from_state: S, result: StepResult<S>, attempt_count: usize) {
        match result {
            StepResult::Transitioned(new_state) => {
                let transition_record = StateTransition {
                    from: from_state,
                    to: new_state.clone(),
                    timestamp: Utc::now(),
                    attempt: attempt_count,
                };
                self.history = self.history.record(transition_record);
                self.current = new_state;
                self.attempt_count = 0;
            }
            StepResult::Retry { .. } => {
                self.attempt_count += 1;
            }
            StepResult::Aborted { error_state, .. } => {
                self.current = error_state;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Guard;
    use crate::effects::transition::{Transition, TransitionResult};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum WorkflowState {
        Initial,
        Processing,
        Complete,
        Failed,
    }

    impl State for WorkflowState {
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

    #[derive(Clone)]
    struct TestEnv {
        _should_succeed: bool,
    }

    #[tokio::test]
    async fn simple_transition_succeeds() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: Arc::new(|| pure(TransitionResult::Success(WorkflowState::Processing)).boxed()),
        };

        machine.add_transition(transition);

        let env = TestEnv {
            _should_succeed: true,
        };
        let (from, result, attempt) = machine.step().run(&env).await.unwrap();
        machine.apply_result(from, result, attempt);

        assert_eq!(machine.current_state(), &WorkflowState::Processing);
        assert_eq!(machine.history().transitions().len(), 1);
    }

    #[tokio::test]
    async fn guard_blocks_transition() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let guard = Guard::new(|s: &WorkflowState| s.is_final());

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: Some(guard),
            action: Arc::new(|| pure(TransitionResult::Success(WorkflowState::Processing)).boxed()),
        };

        machine.add_transition(transition);

        let env = TestEnv {
            _should_succeed: true,
        };
        let result = machine.step().run(&env).await;

        // Should fail because Initial is not final
        assert!(result.is_err());
        assert_eq!(machine.current_state(), &WorkflowState::Initial);
    }

    #[tokio::test]
    async fn retry_increments_attempt_count() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: Arc::new(|| {
                pure(TransitionResult::Retry {
                    feedback: "Not ready yet".to_string(),
                    current_state: WorkflowState::Initial,
                })
                .boxed()
            }),
        };

        machine.add_transition(transition);

        let env = TestEnv {
            _should_succeed: false,
        };
        let (from, result, attempt) = machine.step().run(&env).await.unwrap();

        match &result {
            StepResult::Retry { attempts, .. } => assert_eq!(*attempts, 1),
            _ => panic!("Expected Retry result"),
        }
        machine.apply_result(from, result, attempt);

        // Second attempt
        let (from2, result2, attempt2) = machine.step().run(&env).await.unwrap();
        match &result2 {
            StepResult::Retry { attempts, .. } => assert_eq!(*attempts, 2),
            _ => panic!("Expected Retry result"),
        }
        machine.apply_result(from2, result2, attempt2);
    }

    #[tokio::test]
    async fn effectful_action_with_environment() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: Arc::new(|| {
                from_fn(|env: &TestEnv| {
                    if env._should_succeed {
                        Ok(TransitionResult::Success(WorkflowState::Processing))
                    } else {
                        Ok(TransitionResult::Abort {
                            reason: "Environment not ready".to_string(),
                            error_state: WorkflowState::Failed,
                        })
                    }
                })
                .boxed()
            }),
        };

        machine.add_transition(transition);

        let env = TestEnv {
            _should_succeed: true,
        };
        let (from, result, attempt) = machine.step().run(&env).await.unwrap();

        assert!(matches!(result, StepResult::Transitioned(_)));
        machine.apply_result(from, result, attempt);
        assert_eq!(machine.current_state(), &WorkflowState::Processing);
    }

    #[tokio::test]
    async fn abort_changes_state() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        let transition = Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: Arc::new(|| {
                pure(TransitionResult::Abort {
                    reason: "Something went wrong".to_string(),
                    error_state: WorkflowState::Failed,
                })
                .boxed()
            }),
        };

        machine.add_transition(transition);

        let env = TestEnv {
            _should_succeed: false,
        };
        let (from, result, attempt) = machine.step().run(&env).await.unwrap();

        match &result {
            StepResult::Aborted { error_state, .. } => {
                assert_eq!(*error_state, WorkflowState::Failed);
            }
            _ => panic!("Expected Aborted result"),
        }
        machine.apply_result(from, result, attempt);
        assert_eq!(machine.current_state(), &WorkflowState::Failed);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::effects::transition::{Transition, TransitionResult};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum WorkflowState {
        Initial,
        Processing,
        Complete,
    }

    impl State for WorkflowState {
        fn name(&self) -> &str {
            match self {
                Self::Initial => "Initial",
                Self::Processing => "Processing",
                Self::Complete => "Complete",
            }
        }

        fn is_final(&self) -> bool {
            matches!(self, Self::Complete)
        }
    }

    #[derive(Clone)]
    struct TestEnv {
        _should_succeed: bool,
    }

    #[tokio::test]
    async fn multi_step_workflow() {
        let mut machine = StateMachine::new(WorkflowState::Initial);

        // Initial -> Processing
        machine.add_transition(Transition {
            from: WorkflowState::Initial,
            to: WorkflowState::Processing,
            guard: None,
            action: Arc::new(|| pure(TransitionResult::Success(WorkflowState::Processing)).boxed()),
        });

        // Processing -> Complete
        machine.add_transition(Transition {
            from: WorkflowState::Processing,
            to: WorkflowState::Complete,
            guard: None,
            action: Arc::new(|| pure(TransitionResult::Success(WorkflowState::Complete)).boxed()),
        });

        let env = TestEnv {
            _should_succeed: true,
        };

        // First step
        let (from, result, attempt) = machine.step().run(&env).await.unwrap();
        machine.apply_result(from, result, attempt);
        assert_eq!(machine.current_state(), &WorkflowState::Processing);

        // Second step
        let (from2, result2, attempt2) = machine.step().run(&env).await.unwrap();
        machine.apply_result(from2, result2, attempt2);
        assert_eq!(machine.current_state(), &WorkflowState::Complete);
        assert!(machine.is_final());

        // Verify history
        let path = machine.history().get_path();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], &WorkflowState::Initial);
        assert_eq!(path[1], &WorkflowState::Processing);
        assert_eq!(path[2], &WorkflowState::Complete);
    }
}
