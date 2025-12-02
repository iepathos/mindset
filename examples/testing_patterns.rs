//! Testing Patterns
//!
//! This example demonstrates testing strategies for state machines.
//!
//! Key concepts:
//! - Pure guards are trivial to test (no mocking needed)
//! - Mock environments for testing effectful actions
//! - Dependency injection via environment traits
//! - Test-driven state machine development
//!
//! Run with: cargo run --example testing_patterns

use mindset::state_enum;

state_enum! {
    enum DocumentState {
        Draft,
        Review,
        Published,
    }
    final: [Published]
}

// Environment traits for dependency injection
trait NotificationService {
    fn notify(&mut self, message: &str) -> Result<(), String>;
}

trait Storage {
    fn save(&mut self, doc_id: u64) -> Result<(), String>;
}

// Domain entity
struct Document {
    id: u64,
    content: String,
}

// Pure guard - easy to test, no mocking needed
fn can_publish(doc: &Document) -> bool {
    !doc.content.is_empty() && doc.content.len() >= 10
}

// Effectful action - testable via mock environment
fn publish_document<Env>(doc: &Document, env: &mut Env) -> Result<(), String>
where
    Env: NotificationService + Storage,
{
    env.save(doc.id)?;
    env.notify(&format!("Document {} published", doc.id))?;
    Ok(())
}

// Mock environment for testing
struct MockEnv {
    notifications: Vec<String>,
    saved_docs: Vec<u64>,
    should_fail: bool,
}

impl NotificationService for MockEnv {
    fn notify(&mut self, message: &str) -> Result<(), String> {
        if self.should_fail {
            return Err("Notification failed".to_string());
        }
        self.notifications.push(message.to_string());
        Ok(())
    }
}

impl Storage for MockEnv {
    fn save(&mut self, doc_id: u64) -> Result<(), String> {
        if self.should_fail {
            return Err("Storage failed".to_string());
        }
        self.saved_docs.push(doc_id);
        Ok(())
    }
}

fn main() {
    println!("=== Testing Patterns Example ===\n");

    // Test 1: Pure guard testing
    println!("Test 1: Pure Guard Testing (No Mocking Needed)");
    let valid_doc = Document {
        id: 1,
        content: "This is valid content".to_string(),
    };
    let invalid_doc = Document {
        id: 2,
        content: "Short".to_string(),
    };

    assert!(can_publish(&valid_doc), "Valid doc should pass guard");
    assert!(!can_publish(&invalid_doc), "Invalid doc should fail guard");
    println!("  ✓ Valid document passes guard");
    println!("  ✓ Invalid document fails guard\n");

    // Test 2: Mock environment for success case
    println!("Test 2: Mock Environment (Success Case)");
    let doc = Document {
        id: 42,
        content: "Production-ready content".to_string(),
    };
    let mut mock = MockEnv {
        notifications: vec![],
        saved_docs: vec![],
        should_fail: false,
    };

    let result = publish_document(&doc, &mut mock);
    assert!(result.is_ok(), "Publish should succeed");
    assert_eq!(mock.saved_docs.len(), 1, "Document should be saved");
    assert_eq!(mock.notifications.len(), 1, "Notification should be sent");
    println!("  ✓ Document saved successfully");
    println!("  ✓ Notification sent: {:?}", mock.notifications[0]);
    println!();

    // Test 3: Mock environment for failure case
    println!("Test 3: Mock Environment (Failure Case)");
    let mut mock = MockEnv {
        notifications: vec![],
        saved_docs: vec![],
        should_fail: true,
    };

    let result = publish_document(&doc, &mut mock);
    assert!(result.is_err(), "Publish should fail");
    println!("  ✓ Failure case handled correctly");
    println!("  ✓ Error: {:?}\n", result.err());

    println!("Key Takeaways:");
    println!("- Pure guards require no mocking - just call them with test data");
    println!("- Mock environments make effectful actions testable");
    println!("- Environment traits enable dependency injection");
    println!("- Test both success and failure paths easily");

    println!("\n=== Example Complete ===");
}
