//! Checkpoint and resume functionality for state machines.
//!
//! This module provides serialization and deserialization capabilities for state machines,
//! enabling long-running workflows to survive process restarts and infrastructure failures.

use crate::core::{State, StateHistory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod error;

pub use error::CheckpointError;

/// Version identifier for checkpoint format
pub const CHECKPOINT_VERSION: u32 = 1;

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

/// Serializable checkpoint of state machine state.
/// Does NOT include transition actions (not serializable).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
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
