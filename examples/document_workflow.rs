//! Document Approval Workflow
//!
//! This example demonstrates a multi-stage approval workflow with guards and actions.
//!
//! Key concepts:
//! - Multi-stage linear workflow (Draft -> Review -> Approved -> Published)
//! - Guards control transitions (validation rules)
//! - Actions perform side effects (audit logging)
//! - Final state (Published is terminal)
//!
//! Run with: cargo run --example document_workflow

use mindset::builder::{StateMachineBuilder, TransitionBuilder};
use mindset::state_enum;

state_enum! {
    enum DocState {
        Draft,
        Review,
        Approved,
        Published,
    }
    final: [Published]
}

// Document entity
struct Document {
    id: u64,
    content: String,
    word_count: usize,
}

// Environment trait for audit logging
trait AuditLog {
    fn log_transition(&mut self, doc_id: u64, from: DocState, to: DocState);
}

// Pure guards - validation logic
fn can_submit_for_review(doc: &Document) -> bool {
    !doc.content.is_empty() && doc.word_count >= 100
}

fn can_approve(doc: &Document) -> bool {
    doc.word_count <= 5000
}

// Effectful actions
fn submit_for_review<Env>(doc: &Document, env: &mut Env)
where
    Env: AuditLog,
{
    env.log_transition(doc.id, DocState::Draft, DocState::Review);
}

fn approve_document<Env>(doc: &Document, env: &mut Env)
where
    Env: AuditLog,
{
    env.log_transition(doc.id, DocState::Review, DocState::Approved);
}

fn publish_document<Env>(doc: &Document, env: &mut Env)
where
    Env: AuditLog,
{
    env.log_transition(doc.id, DocState::Approved, DocState::Published);
}

// Simple audit log implementation
struct SimpleAuditLog;

impl AuditLog for SimpleAuditLog {
    fn log_transition(&mut self, doc_id: u64, from: DocState, to: DocState) {
        println!(
            "  [Audit] Document {} transitioned from {:?} to {:?}",
            doc_id, from, to
        );
    }
}

fn main() {
    println!("=== Document Approval Workflow ===\n");

    // Create state machine
    let _machine = StateMachineBuilder::<DocState, ()>::new()
        .initial(DocState::Draft)
        .add_transition(
            TransitionBuilder::new()
                .from(DocState::Draft)
                .to(DocState::Review)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(DocState::Review)
                .to(DocState::Approved)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(DocState::Approved)
                .to(DocState::Published)
                .succeeds()
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    println!("Created document workflow state machine");
    println!("States: Draft -> Review -> Approved -> Published\n");

    // Create document and audit log
    let doc = Document {
        id: 123,
        content: "Lorem ipsum dolor sit amet...".to_string(),
        word_count: 250,
    };
    let mut audit = SimpleAuditLog;

    // Simulate workflow
    println!("Processing document {}:", doc.id);
    println!("  Word count: {}", doc.word_count);
    println!();

    if can_submit_for_review(&doc) {
        println!("Step 1: Submit for Review");
        submit_for_review(&doc, &mut audit);
        println!("  ✓ Document meets minimum requirements (100+ words)\n");

        if can_approve(&doc) {
            println!("Step 2: Approve");
            approve_document(&doc, &mut audit);
            println!("  ✓ Document within maximum length (5000 words)\n");

            println!("Step 3: Publish");
            publish_document(&doc, &mut audit);
            println!("  ✓ Document published successfully\n");
        }
    }

    println!("Key Takeaways:");
    println!("- Linear workflow with clear progression");
    println!("- Guards enforce business rules (word count)");
    println!("- Actions perform side effects (audit logging)");
    println!("- Final state (Published) is terminal");

    println!("\n=== Example Complete ===");
}
