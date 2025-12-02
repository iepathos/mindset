//! Account Management
//!
//! This example demonstrates account lifecycle with validation guards.
//!
//! Key concepts:
//! - Account states (Active -> Suspended -> Closed)
//! - Validation guards for state transitions
//! - Balance requirements and business rules
//! - Database persistence via environment pattern
//!
//! Run with: cargo run --example account_management

use mindset::builder::{StateMachineBuilder, TransitionBuilder};
use mindset::core::State;
use serde::{Deserialize, Serialize};

// Account states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum AccountStatus {
    Active,
    Suspended,
    Closed,
}

impl State for AccountStatus {
    fn name(&self) -> &str {
        match self {
            Self::Active => "Active",
            Self::Suspended => "Suspended",
            Self::Closed => "Closed",
        }
    }

    fn is_final(&self) -> bool {
        matches!(self, Self::Closed)
    }
}

// Account entity
struct Account {
    id: u64,
    balance: f64,
    violations: u32,
}

// Environment trait
trait AccountRepository {
    fn persist(&mut self, account: &Account) -> Result<(), String>;
}

// Pure guards - validation logic
fn can_suspend(account: &Account) -> bool {
    account.violations >= 3
}

fn can_reactivate(account: &Account) -> bool {
    account.violations < 3 && account.balance >= 0.0
}

fn can_close(account: &Account) -> bool {
    account.balance == 0.0
}

// Effectful actions
fn suspend_account<Env>(account: &mut Account, env: &mut Env) -> Result<(), String>
where
    Env: AccountRepository,
{
    println!(
        "  Suspending account {} due to {} violations",
        account.id, account.violations
    );
    env.persist(account)?;
    Ok(())
}

fn reactivate_account<Env>(account: &mut Account, env: &mut Env) -> Result<(), String>
where
    Env: AccountRepository,
{
    println!("  Reactivating account {}", account.id);
    account.violations = 0;
    env.persist(account)?;
    Ok(())
}

fn close_account<Env>(account: &mut Account, env: &mut Env) -> Result<(), String>
where
    Env: AccountRepository,
{
    if !can_close(account) {
        return Err(format!(
            "Cannot close account with non-zero balance: ${}",
            account.balance
        ));
    }
    println!("  Closing account {}", account.id);
    env.persist(account)?;
    Ok(())
}

// Mock repository
struct MockRepository {
    persisted_count: usize,
}

impl AccountRepository for MockRepository {
    fn persist(&mut self, account: &Account) -> Result<(), String> {
        println!(
            "  [DB] Persisted account {} (balance: ${})",
            account.id, account.balance
        );
        self.persisted_count += 1;
        Ok(())
    }
}

fn main() {
    println!("=== Account Management Example ===\n");

    // Create state machine
    let _machine = StateMachineBuilder::<AccountStatus, ()>::new()
        .initial(AccountStatus::Active)
        .add_transition(
            TransitionBuilder::new()
                .from(AccountStatus::Active)
                .to(AccountStatus::Suspended)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(AccountStatus::Suspended)
                .to(AccountStatus::Active)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(AccountStatus::Active)
                .to(AccountStatus::Closed)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(AccountStatus::Suspended)
                .to(AccountStatus::Closed)
                .succeeds()
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    println!("Account management state machine created");
    println!("States: Active <-> Suspended -> Closed\n");

    let mut repo = MockRepository { persisted_count: 0 };

    // Scenario 1: Suspend account due to violations
    println!("Scenario 1: Suspend Account");
    let mut account1 = Account {
        id: 1001,
        balance: 100.0,
        violations: 5,
    };

    if can_suspend(&account1) {
        println!(
            "  Guard passed: Account has {} violations (>= 3)",
            account1.violations
        );
        suspend_account(&mut account1, &mut repo).unwrap();
        println!("  ✓ Account suspended\n");
    }

    // Scenario 2: Reactivate account
    println!("Scenario 2: Reactivate Account");
    let mut account2 = Account {
        id: 1002,
        balance: 50.0,
        violations: 2,
    };

    if can_reactivate(&account2) {
        println!("  Guard passed: Violations < 3 and balance >= 0");
        reactivate_account(&mut account2, &mut repo).unwrap();
        println!("  ✓ Account reactivated\n");
    }

    // Scenario 3: Close account with zero balance
    println!("Scenario 3: Close Account (Success)");
    let mut account3 = Account {
        id: 1003,
        balance: 0.0,
        violations: 0,
    };

    if can_close(&account3) {
        println!("  Guard passed: Balance is zero");
        close_account(&mut account3, &mut repo).unwrap();
        println!("  ✓ Account closed\n");
    }

    // Scenario 4: Try to close account with non-zero balance
    println!("Scenario 4: Close Account (Failure)");
    let account4 = Account {
        id: 1004,
        balance: 25.50,
        violations: 0,
    };

    if !can_close(&account4) {
        println!(
            "  Guard failed: Balance is not zero (${:.2})",
            account4.balance
        );
        println!("  ✗ Cannot close account\n");
    }

    println!("Total database operations: {}", repo.persisted_count);

    println!("\nKey Takeaways:");
    println!("- Guards enforce business rules (balance, violations)");
    println!("- Multiple paths to final state (Active/Suspended -> Closed)");
    println!("- Bidirectional transitions (Active <-> Suspended)");
    println!("- Clear error messages when guards fail");

    println!("\n=== Example Complete ===");
}
