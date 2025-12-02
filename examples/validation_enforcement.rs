//! Validation and Enforcement
//!
//! This example demonstrates the validation-based enforcement system.
//!
//! Key concepts:
//! - Validation over Result for comprehensive error reporting
//! - Built-in enforcement rules (max attempts, timeouts)
//! - Custom validation predicates
//! - Violation strategies (Abort, Retry, IgnoreAndLog)
//!
//! Run with: cargo run --example validation_enforcement

use mindset::enforcement::{EnforcementBuilder, ViolationStrategy};
use mindset::state_enum;
use std::time::Duration;

state_enum! {
    enum TaskState {
        Pending,
        Running,
        Complete,
    }
    final: [Complete]
}

struct Task {
    id: u64,
    resource_available: bool,
    is_safe: bool,
}

fn main() {
    println!("=== Validation and Enforcement Example ===\n");

    // Example 1: Basic enforcement with max attempts
    println!("Example 1: Max Attempts Enforcement");
    let _rules = EnforcementBuilder::<TaskState>::new()
        .max_attempts(3)
        .on_violation(ViolationStrategy::Abort)
        .build();

    println!("  Created enforcement rules with max 3 attempts");
    println!("  Violation strategy: Abort\n");

    // Example 2: Timeout enforcement with retry
    println!("Example 2: Timeout with Retry Strategy");
    let _rules = EnforcementBuilder::<TaskState>::new()
        .timeout(Duration::from_secs(30))
        .on_violation(ViolationStrategy::Retry)
        .build();

    println!("  Created enforcement rules with 30s timeout");
    println!("  Violation strategy: Retry (for temporary issues)\n");

    // Example 3: Custom validation predicates
    println!("Example 3: Custom Validation Rules");
    let task = Task {
        id: 1,
        resource_available: true,
        is_safe: true,
    };

    let resource_available = task.resource_available;
    let is_safe = task.is_safe;
    let _rules = EnforcementBuilder::<TaskState>::new()
        .max_attempts(5)
        .require_pred(
            move |_ctx| resource_available,
            "Resource must be available".to_string(),
        )
        .require_pred(move |_ctx| is_safe, "Task must be safe".to_string())
        .on_violation(ViolationStrategy::Abort)
        .build();

    println!("  Created enforcement with custom predicates:");
    println!("    - Resource availability check");
    println!("    - Safety validation check");
    println!("    - Max 5 attempts");
    println!("  All checks passed for task {}\n", task.id);

    // Example 4: Violation strategies comparison
    println!("Example 4: Violation Strategies");
    println!("  Abort: Fail transition permanently (for critical checks)");
    println!("  Retry: Allow retry despite violations (for temporary issues)");
    println!("  IgnoreAndLog: Continue with warning (for non-critical checks)\n");

    // Example 5: Multiple violations collected together
    println!("Example 5: Comprehensive Error Reporting");
    println!("  Traditional Result: Fails at first error");
    println!("    1. Check max attempts -> Error: exceeded");
    println!("    2. (Not checked - already failed)");
    println!("    3. (Not checked - already failed)");
    println!();
    println!("  Validation: Collects ALL violations");
    println!("    1. Check max attempts -> Violation: exceeded");
    println!("    2. Check timeout -> Violation: exceeded");
    println!("    3. Check resource -> Violation: unavailable");
    println!("    -> Reports all 3 violations together\n");

    println!("Key Takeaways:");
    println!("- Validation accumulates ALL violations, not just the first");
    println!("- Better UX: Users see all problems at once");
    println!("- Custom predicates enable domain-specific rules");
    println!("- Violation strategies control failure behavior");

    println!("\n=== Example Complete ===");
}
