//! Validation-based enforcement system for state transitions.
//!
//! This module implements a policy enforcement system using Stillwater's
//! `Validation` type to accumulate ALL violations instead of fail-fast behavior.
//!
//! # Philosophy
//!
//! Following Stillwater's philosophy: "Don't stop at first error - collect them all!"
//!
//! Traditional validation with `Result` stops at the first error, frustrating users
//! who must fix errors one at a time. The `Validation` type accumulates all errors,
//! providing comprehensive feedback in a single pass.
//!
//! # Example
//!
//! ```rust
//! use mindset::enforcement::{EnforcementBuilder, EnforcementRules, ViolationStrategy};
//! use std::time::Duration;
//!
//! # use mindset::core::State;
//! # use serde::{Deserialize, Serialize};
//! # #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
//! # enum TestState { Initial, Processing }
//! # impl State for TestState {
//! #     fn name(&self) -> &str { "TestState" }
//! #     fn is_final(&self) -> bool { false }
//! # }
//!
//! let rules: EnforcementRules<TestState> = EnforcementBuilder::new()
//!     .max_attempts(3)
//!     .timeout(Duration::from_secs(30))
//!     .on_violation(ViolationStrategy::Abort)
//!     .build();
//! ```

pub mod builder;
pub mod context;
pub mod rules;
pub mod violations;

// Re-export commonly used types
pub use builder::EnforcementBuilder;
pub use context::TransitionContext;
pub use rules::EnforcementRules;
pub use violations::{ViolationError, ViolationStrategy};
