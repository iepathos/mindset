//! MapReduce Workflow
//!
//! This example demonstrates MapReduce workflow patterns with state machines.
//!
//! Key concepts:
//! - Multi-phase workflow (Map -> Reduce -> Complete)
//! - Parallel processing patterns
//! - Aggregation of intermediate results
//! - Workflow orchestration with state machines
//!
//! Run with: cargo run --example mapreduce_workflow

use std::collections::HashMap;

// MapReduce workflow phases
#[derive(Debug, Clone, Copy, PartialEq)]
enum WorkflowPhase {
    Initializing,
    Mapping,
    Reducing,
    Complete,
}

// Work item for processing
#[derive(Debug, Clone)]
struct WorkItem {
    id: u64,
    data: String,
}

// Intermediate result from map phase
#[derive(Debug, Clone)]
struct MapResult {
    key: String,
    value: i32,
}

// Final aggregated result
#[derive(Debug)]
#[allow(dead_code)]
struct ReduceResult {
    key: String,
    total: i32,
}

// MapReduce workflow state
struct Workflow {
    phase: WorkflowPhase,
    items: Vec<WorkItem>,
    map_results: Vec<MapResult>,
    reduce_results: Vec<ReduceResult>,
}

impl Workflow {
    fn new(items: Vec<WorkItem>) -> Self {
        Self {
            phase: WorkflowPhase::Initializing,
            items,
            map_results: vec![],
            reduce_results: vec![],
        }
    }

    fn initialize(&mut self) {
        println!("Initializing workflow with {} items", self.items.len());
        self.phase = WorkflowPhase::Mapping;
    }

    fn map_phase(&mut self) {
        println!("\n=== Map Phase ===");
        println!("Processing {} items in parallel...\n", self.items.len());

        // Simulate parallel map operations
        for item in &self.items {
            let results = self.map_item(item);
            println!(
                "  Item {}: Generated {} map results",
                item.id,
                results.len()
            );
            self.map_results.extend(results);
        }

        println!(
            "\nMap phase complete: {} total results",
            self.map_results.len()
        );
        self.phase = WorkflowPhase::Reducing;
    }

    fn map_item(&self, item: &WorkItem) -> Vec<MapResult> {
        // Example: word count mapping
        item.data
            .split_whitespace()
            .map(|word| MapResult {
                key: word.to_lowercase(),
                value: 1,
            })
            .collect()
    }

    fn reduce_phase(&mut self) {
        println!("\n=== Reduce Phase ===");
        println!("Aggregating {} map results...\n", self.map_results.len());

        // Group by key
        let mut groups: HashMap<String, Vec<i32>> = HashMap::new();
        for result in &self.map_results {
            groups
                .entry(result.key.clone())
                .or_default()
                .push(result.value);
        }

        // Reduce each group
        for (key, values) in groups {
            let total: i32 = values.iter().sum();
            println!("  '{}': {} occurrences", key, total);
            self.reduce_results.push(ReduceResult { key, total });
        }

        println!(
            "\nReduce phase complete: {} unique keys",
            self.reduce_results.len()
        );
        self.phase = WorkflowPhase::Complete;
    }

    fn is_complete(&self) -> bool {
        matches!(self.phase, WorkflowPhase::Complete)
    }
}

fn main() {
    println!("=== MapReduce Workflow Example ===\n");

    // Create sample work items
    let items = vec![
        WorkItem {
            id: 1,
            data: "The quick brown fox".to_string(),
        },
        WorkItem {
            id: 2,
            data: "The lazy dog sleeps".to_string(),
        },
        WorkItem {
            id: 3,
            data: "The fox jumps high".to_string(),
        },
    ];

    // Create workflow
    let mut workflow = Workflow::new(items);

    // Execute workflow phases
    workflow.initialize();

    // Map phase
    workflow.map_phase();

    // Reduce phase
    workflow.reduce_phase();

    // Verify completion
    assert!(workflow.is_complete(), "Workflow should be complete");

    println!("\n=== Workflow Summary ===");
    println!("Total items processed: {}", workflow.items.len());
    println!("Map results generated: {}", workflow.map_results.len());
    println!("Unique words counted: {}", workflow.reduce_results.len());
    println!("Status: {:?}", workflow.phase);

    println!("\nKey Takeaways:");
    println!("- MapReduce splits work into parallel map phase");
    println!("- Intermediate results are grouped by key");
    println!("- Reduce phase aggregates grouped results");
    println!("- State machine orchestrates workflow phases");

    println!("\n=== Example Complete ===");
}
