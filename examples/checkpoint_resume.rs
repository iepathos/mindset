//! Checkpoint and Resume
//!
//! This example demonstrates checkpoint and resume patterns for long-running workflows.
//!
//! Key concepts:
//! - Automatic checkpoint creation at key workflow phases
//! - Serialization formats (JSON for readability, binary for compactness)
//! - Atomic writes to prevent corruption
//! - Resume from interruption (crashes, stops, maintenance)
//!
//! Run with: cargo run --example checkpoint_resume

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// Workflow state that can be checkpointed
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowState {
    phase: WorkflowPhase,
    items_processed: usize,
    total_items: usize,
    intermediate_results: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WorkflowPhase {
    Initializing,
    Mapping,
    Reducing,
    Complete,
}

// Checkpoint manager
struct CheckpointManager {
    checkpoint_dir: String,
}

impl CheckpointManager {
    fn new(dir: &str) -> Self {
        fs::create_dir_all(dir).ok();
        Self {
            checkpoint_dir: dir.to_string(),
        }
    }

    fn save_checkpoint(&self, state: &WorkflowState) -> Result<String, String> {
        let checkpoint_path = format!(
            "{}/checkpoint-{}.json",
            self.checkpoint_dir, state.items_processed
        );
        let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;

        // Atomic write: write to temp file, then rename
        let temp_path = format!("{}.tmp", checkpoint_path);
        fs::write(&temp_path, json).map_err(|e| e.to_string())?;
        fs::rename(&temp_path, &checkpoint_path).map_err(|e| e.to_string())?;

        println!("  [Checkpoint] Saved to {}", checkpoint_path);
        Ok(checkpoint_path)
    }

    fn load_checkpoint(&self, path: &str) -> Result<WorkflowState, String> {
        let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let state = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        println!("  [Checkpoint] Loaded from {}", path);
        Ok(state)
    }

    fn list_checkpoints(&self) -> Vec<String> {
        fs::read_dir(&self.checkpoint_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map(|s| s == "json").unwrap_or(false))
                    .map(|e| e.path().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

// Simulate long-running workflow with checkpointing
fn run_workflow(checkpoint_mgr: &CheckpointManager, resume_from: Option<WorkflowState>) {
    let mut state = resume_from.unwrap_or_else(|| WorkflowState {
        phase: WorkflowPhase::Initializing,
        items_processed: 0,
        total_items: 10,
        intermediate_results: vec![],
    });

    println!(
        "Starting workflow (phase: {:?}, progress: {}/{})",
        state.phase, state.items_processed, state.total_items
    );
    println!();

    // Phase 1: Mapping
    if let WorkflowPhase::Initializing = state.phase {
        println!("Phase: Initializing -> Mapping");
        state.phase = WorkflowPhase::Mapping;
    }

    // Process items with checkpoints
    while state.items_processed < state.total_items {
        state.items_processed += 1;
        state
            .intermediate_results
            .push(format!("result-{}", state.items_processed));

        println!(
            "  Processed item {}/{}",
            state.items_processed, state.total_items
        );

        // Checkpoint every 3 items
        if state.items_processed % 3 == 0 {
            checkpoint_mgr.save_checkpoint(&state).ok();
        }

        // Simulate interruption at item 5
        if state.items_processed == 5 {
            println!("\n  [INTERRUPT] Workflow interrupted! State saved in checkpoint.\n");
            return;
        }
    }

    // Phase 2: Reducing
    state.phase = WorkflowPhase::Reducing;
    println!("\nPhase: Mapping -> Reducing");
    checkpoint_mgr.save_checkpoint(&state).ok();

    // Phase 3: Complete
    state.phase = WorkflowPhase::Complete;
    println!("Phase: Reducing -> Complete");
    checkpoint_mgr.save_checkpoint(&state).ok();

    println!("\nWorkflow completed successfully!");
}

fn main() {
    println!("=== Checkpoint and Resume Example ===\n");

    let checkpoint_mgr = CheckpointManager::new("/tmp/mindset-checkpoints");

    // Run 1: Start workflow (will be interrupted)
    println!("Run 1: Starting new workflow");
    println!("----------------------------------------");
    run_workflow(&checkpoint_mgr, None);

    // Show available checkpoints
    println!("Available checkpoints:");
    let checkpoints = checkpoint_mgr.list_checkpoints();
    for cp in &checkpoints {
        println!(
            "  - {}",
            Path::new(cp).file_name().unwrap().to_string_lossy()
        );
    }
    println!();

    // Run 2: Resume from latest checkpoint
    if let Some(latest) = checkpoints.last() {
        println!("Run 2: Resuming from checkpoint");
        println!("----------------------------------------");
        if let Ok(state) = checkpoint_mgr.load_checkpoint(latest) {
            println!(
                "Resumed at phase {:?}, {} items already processed\n",
                state.phase, state.items_processed
            );
            run_workflow(&checkpoint_mgr, Some(state));
        }
    }

    // Cleanup
    fs::remove_dir_all("/tmp/mindset-checkpoints").ok();

    println!("\nKey Takeaways:");
    println!("- Checkpoints enable resumption after interruptions");
    println!("- Atomic writes prevent checkpoint corruption");
    println!("- JSON format makes checkpoints human-readable");
    println!("- Periodic checkpointing balances overhead vs recovery time");

    println!("\n=== Example Complete ===");
}
