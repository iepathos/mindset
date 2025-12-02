---
number: 004
title: Checkpoint and Resume
category: storage
priority: high
status: draft
dependencies: [001, 002, 003]
created: 2025-12-01
---

# Specification 004: Checkpoint and Resume

**Category**: storage
**Priority**: high
**Status**: draft
**Dependencies**: Specification 001, 002, 003

## Context

Long-running state machine workflows need to survive:
- Process crashes or restarts
- Deployments and updates
- Infrastructure failures
- Intentional pauses for human review

Without checkpointing, all progress is lost. With checkpointing, machines can resume exactly where they left off, preserving state history and metadata.

This is critical for Platypus AI workflows that may run for hours or days.

## Objective

Implement serialization and deserialization of state machines to JSON and binary formats, enabling checkpoint creation and resumption from saved state with full history preservation.

## Requirements

### Functional Requirements

- **Checkpoint Creation**: Serialize complete state machine state
- **Checkpoint Resume**: Deserialize and rebuild state machine
- **History Preservation**: Maintain complete transition history
- **Metadata Tracking**: Preserve attempt counts, timestamps, duration
- **Multiple Formats**: Support JSON (human-readable) and binary (compact)
- **Versioning**: Include checkpoint version for future compatibility
- **Validation**: Verify checkpoint integrity on load

### Non-Functional Requirements

- **Correctness**: Resumed machine behaves identically to pre-checkpoint
- **Efficiency**: Binary format < 50% size of JSON
- **Human-Readable**: JSON format for debugging and inspection
- **Forward Compatible**: Design for schema evolution
- **Atomic Writes**: Checkpoint writes should be atomic

## Acceptance Criteria

- [ ] `Checkpoint<S>` struct with all necessary state
- [ ] `to_json()` serializes machine to JSON string
- [ ] `to_binary()` serializes machine to binary Vec<u8>
- [ ] `from_json()` deserializes and rebuilds machine
- [ ] `from_binary()` deserializes from binary format
- [ ] History fully preserved across checkpoint/resume
- [ ] Metadata (timestamps, attempts) preserved
- [ ] Checkpoint includes version identifier
- [ ] Roundtrip tests (serialize → deserialize → identical)
- [ ] Error handling for corrupted checkpoints
- [ ] Documentation with examples

## Technical Details

### Implementation Approach

Separate checkpoint data (serializable) from runtime behavior (transitions). Checkpoint contains only data; transitions are reconstructed on resume.

### Checkpoint Type

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::core::{State, StateHistory};

/// Version identifier for checkpoint format
const CHECKPOINT_VERSION: u32 = 1;

/// Serializable checkpoint of state machine state.
/// Does NOT include transition actions (not serializable).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint<S: State> {
    /// Checkpoint format version
    pub version: u32,

    /// Unique checkpoint identifier
    pub id: String,

    /// When checkpoint was created
    pub timestamp: DateTime<Utc>,

    /// Initial state of the machine
    pub initial_state: S,

    /// Current state of the machine
    pub current_state: S,

    /// Complete transition history
    pub history: StateHistory<S>,

    /// Machine metadata
    pub metadata: MachineMetadata,
}

/// Metadata tracked by state machine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachineMetadata {
    /// When machine was created
    pub created_at: DateTime<Utc>,

    /// Last update time
    pub updated_at: DateTime<Utc>,

    /// Current attempt count for active transition
    pub current_attempt: usize,

    /// Total attempts per transition (transition name -> count)
    pub total_attempts: HashMap<String, usize>,
}

impl Default for MachineMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            current_attempt: 0,
            total_attempts: HashMap::new(),
        }
    }
}
```

### Checkpoint Creation

```rust
use uuid::Uuid;

impl<S: State, Env> StateMachine<S, Env> {
    /// Create a checkpoint of current machine state.
    /// Pure function - does not modify machine.
    pub fn checkpoint(&self) -> Checkpoint<S> {
        Checkpoint {
            version: CHECKPOINT_VERSION,
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            initial_state: self.initial.clone(),
            current_state: self.current.clone(),
            history: self.history.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, CheckpointError> {
        let checkpoint = self.checkpoint();
        serde_json::to_string_pretty(&checkpoint)
            .map_err(|e| CheckpointError::SerializationFailed(e.to_string()))
    }

    /// Serialize to binary format
    pub fn to_binary(&self) -> Result<Vec<u8>, CheckpointError> {
        let checkpoint = self.checkpoint();
        bincode::serialize(&checkpoint)
            .map_err(|e| CheckpointError::SerializationFailed(e.to_string()))
    }
}
```

### Checkpoint Resume

```rust
impl<S: State, Env> StateMachine<S, Env> {
    /// Create state machine from checkpoint.
    /// Transitions must be provided (not serializable).
    pub fn from_checkpoint(
        checkpoint: Checkpoint<S>,
        transitions: Vec<Transition<S, Env>>,
    ) -> Result<Self, CheckpointError> {
        // Validate version
        if checkpoint.version > CHECKPOINT_VERSION {
            return Err(CheckpointError::UnsupportedVersion {
                found: checkpoint.version,
                supported: CHECKPOINT_VERSION,
            });
        }

        Ok(Self {
            initial: checkpoint.initial_state,
            current: checkpoint.current_state,
            transitions,
            history: checkpoint.history,
            metadata: checkpoint.metadata,
        })
    }

    /// Deserialize from JSON string
    pub fn from_json(
        json: &str,
        transitions: Vec<Transition<S, Env>>,
    ) -> Result<Self, CheckpointError> {
        let checkpoint: Checkpoint<S> = serde_json::from_str(json)
            .map_err(|e| CheckpointError::DeserializationFailed(e.to_string()))?;

        Self::from_checkpoint(checkpoint, transitions)
    }

    /// Deserialize from binary format
    pub fn from_binary(
        bytes: &[u8],
        transitions: Vec<Transition<S, Env>>,
    ) -> Result<Self, CheckpointError> {
        let checkpoint: Checkpoint<S> = bincode::deserialize(bytes)
            .map_err(|e| CheckpointError::DeserializationFailed(e.to_string()))?;

        Self::from_checkpoint(checkpoint, transitions)
    }
}
```

### Error Types

```rust
use thiserror::Error;

/// Errors that can occur during checkpoint operations
#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("Unsupported checkpoint version {found}, supported: {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },

    #[error("Checkpoint validation failed: {0}")]
    ValidationFailed(String),
}
```

### Update StateMachine with Metadata

```rust
// Add metadata field to StateMachine
pub struct StateMachine<S: State, Env> {
    initial: S,
    current: S,
    transitions: Vec<Transition<S, Env>>,
    history: StateHistory<S>,
    metadata: MachineMetadata,  // NEW
}

impl<S: State, Env> StateMachine<S, Env> {
    pub fn new(initial: S) -> Self {
        Self {
            initial: initial.clone(),
            current: initial,
            transitions: Vec::new(),
            history: StateHistory::new(),
            metadata: MachineMetadata::default(),
        }
    }

    /// Update metadata after transition
    fn update_metadata(&mut self, transition_name: &str) {
        self.metadata.updated_at = Utc::now();
        *self.metadata.total_attempts.entry(transition_name.to_string()).or_insert(0) += 1;
    }
}
```

### Module Structure

```
mindset/
├── src/
│   ├── checkpoint/
│   │   ├── mod.rs
│   │   ├── checkpoint.rs    # Checkpoint type
│   │   ├── metadata.rs      # MachineMetadata
│   │   └── error.rs         # CheckpointError
```

## Dependencies

- **Prerequisites**: Specification 001, 002, 003
- **Affected Components**: StateMachine (add metadata field)
- **External Dependencies**:
  - `serde_json = "1.0"`
  - `bincode = "1.3"`
  - `uuid = { version = "1.0", features = ["v4", "serde"] }`

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_serializes_to_json() {
        let machine = create_test_machine();
        let json = machine.to_json().unwrap();

        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());

        // Verify contains expected fields
        assert!(json.contains("version"));
        assert!(json.contains("current_state"));
        assert!(json.contains("history"));
    }

    #[test]
    fn checkpoint_roundtrip_preserves_state() {
        let mut machine1 = create_test_machine();

        // Execute some transitions
        let env = TestEnv { should_succeed: true };
        machine1.step().run(&env).await.unwrap();
        machine1.step().run(&env).await.unwrap();

        // Checkpoint and restore
        let json = machine1.to_json().unwrap();
        let transitions = create_test_transitions();
        let machine2 = StateMachine::from_json(&json, transitions).unwrap();

        // Verify state preserved
        assert_eq!(machine1.current_state(), machine2.current_state());
        assert_eq!(machine1.initial, machine2.initial);
        assert_eq!(
            machine1.history().transitions().len(),
            machine2.history().transitions().len()
        );
    }

    #[test]
    fn binary_format_smaller_than_json() {
        let machine = create_test_machine();

        let json = machine.to_json().unwrap();
        let binary = machine.to_binary().unwrap();

        // Binary should be significantly smaller
        assert!(binary.len() < json.len() / 2);
    }

    #[test]
    fn resumed_machine_can_continue_execution() {
        let mut machine1 = create_test_machine();
        let env = TestEnv { should_succeed: true };

        // Execute first transition
        machine1.step().run(&env).await.unwrap();
        assert_eq!(machine1.current_state(), &TestState::Processing);

        // Checkpoint
        let json = machine1.to_json().unwrap();

        // Resume
        let transitions = create_test_transitions();
        let mut machine2 = StateMachine::from_json(&json, transitions).unwrap();

        // Should be able to continue from where we left off
        machine2.step().run(&env).await.unwrap();
        assert_eq!(machine2.current_state(), &TestState::Complete);
    }

    #[test]
    fn unsupported_version_returns_error() {
        let mut checkpoint = create_test_checkpoint();
        checkpoint.version = 999;

        let json = serde_json::to_string(&checkpoint).unwrap();
        let result = StateMachine::<TestState, ()>::from_json(&json, vec![]);

        assert!(matches!(
            result,
            Err(CheckpointError::UnsupportedVersion { .. })
        ));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn checkpoint_resume_preserves_history() {
    let mut machine = create_multi_step_machine();
    let env = TestEnv { should_succeed: true };

    // Execute several steps
    for _ in 0..3 {
        machine.step().run(&env).await.unwrap();
    }

    let original_history = machine.history().transitions().to_vec();

    // Checkpoint and resume
    let json = machine.to_json().unwrap();
    let transitions = create_transitions();
    let restored = StateMachine::from_json(&json, transitions).unwrap();

    let restored_history = restored.history().transitions();

    // History should be identical
    assert_eq!(original_history.len(), restored_history.len());
    for (orig, restored) in original_history.iter().zip(restored_history.iter()) {
        assert_eq!(orig.from, restored.from);
        assert_eq!(orig.to, restored.to);
        assert_eq!(orig.attempt, restored.attempt);
    }
}
```

## Documentation Requirements

### Code Documentation

- Document checkpoint/resume workflow
- Explain transition reconstruction requirement
- Show JSON format example

### User Documentation

Create `docs/checkpointing.md`:
- Why checkpointing matters
- When to create checkpoints
- How to restore from checkpoint
- Best practices for checkpoint storage
- Example: pause and resume workflow

### Architecture Updates

Update README.md:
- Checkpoint/resume capability
- Serialization formats supported
- Use cases for long-running workflows

## Implementation Notes

### Why Transitions Not Serialized

Transitions contain closures and effects that cannot be serialized:
```rust
pub action: BoxedEffect<TransitionResult<S>, E, Env>
```

Instead, user must reconstruct transition definitions when resuming:
```rust
let transitions = vec![
    create_initial_to_processing_transition(),
    create_processing_to_complete_transition(),
];

let machine = StateMachine::from_json(&json, transitions)?;
```

This is acceptable because:
- Transition logic is code (should be in version control)
- Checkpoint data is state (needs persistence)
- Clear separation of concerns

### Atomic Checkpoint Writes

For production use, recommend atomic writes:
```rust
use std::fs;
use tempfile::NamedTempFile;

pub fn save_checkpoint_atomic(machine: &StateMachine, path: &Path) -> Result<()> {
    let json = machine.to_json()?;

    // Write to temp file
    let mut temp = NamedTempFile::new_in(path.parent().unwrap())?;
    temp.write_all(json.as_bytes())?;

    // Atomic rename
    temp.persist(path)?;
    Ok(())
}
```

### Version Evolution

Design for future schema changes:
```rust
match checkpoint.version {
    1 => load_v1(checkpoint),
    2 => load_v2(checkpoint),
    _ => Err(UnsupportedVersion),
}
```

## Migration and Compatibility

No migration needed (new feature). Design for forward compatibility:
- Version field enables schema evolution
- Can add fields with `#[serde(default)]`
- Can deprecate fields with `#[serde(skip_serializing_if)]`
