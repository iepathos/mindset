//! Effectful State Machine
//!
//! This example demonstrates effectful state machines using the environment pattern.
//!
//! Key concepts:
//! - Environment pattern for dependency injection
//! - Separation of pure guards from effectful actions
//! - Type-safe effect composition
//! - Testing with mock environments
//!
//! Run with: cargo run --example effectful_state_machine

use mindset::state_enum;

// Define order states
state_enum! {
    enum OrderState {
        Draft,
        Submitted,
        Processing,
        Completed,
    }
    final: [Completed]
}

// Define environment capabilities as traits
trait PaymentProcessor {
    fn charge(&mut self, amount: f64) -> Result<(), String>;
}

trait Logger {
    fn log(&mut self, message: &str);
}

// Order entity
struct Order {
    id: u64,
    total: f64,
}

// Pure guard - no side effects
fn can_submit(order: &Order) -> bool {
    order.total > 0.0
}

// Effectful action - explicit environment usage
fn submit_order<Env>(order: &mut Order, env: &mut Env) -> Result<(), String>
where
    Env: PaymentProcessor + Logger,
{
    env.log(&format!("Submitting order {}", order.id));
    env.charge(order.total)?;
    env.log(&format!("Order {} submitted successfully", order.id));
    Ok(())
}

// Production environment implementation
struct ProductionEnv;

impl PaymentProcessor for ProductionEnv {
    fn charge(&mut self, amount: f64) -> Result<(), String> {
        println!("  [Payment] Charging ${:.2}", amount);
        Ok(())
    }
}

impl Logger for ProductionEnv {
    fn log(&mut self, message: &str) {
        println!("  [Log] {}", message);
    }
}

fn main() {
    println!("=== Effectful State Machine Example ===\n");

    // Create order
    let mut order = Order {
        id: 42,
        total: 99.99,
    };

    // Create environment
    let mut env = ProductionEnv;

    // Check guard
    println!("Checking if order can be submitted...");
    if can_submit(&order) {
        println!("  Guard passed: Order total is positive\n");

        // Execute effectful action
        println!("Executing submission with effects:");
        match submit_order(&mut order, &mut env) {
            Ok(_) => println!("\n  Success!\n"),
            Err(e) => println!("\n  Error: {}\n", e),
        }
    } else {
        println!("  Guard failed: Order total must be positive\n");
    }

    println!("Key Takeaways:");
    println!("- Guards (can_submit) are pure functions with no side effects");
    println!("- Actions (submit_order) are explicit about their effects via environment");
    println!("- Environment traits enable testing with mocks");
    println!("- Type system enforces separation of concerns");

    println!("\n=== Example Complete ===");
}
