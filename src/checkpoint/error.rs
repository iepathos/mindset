//! Checkpoint error types.

use thiserror::Error;

/// Errors that can occur during checkpoint operations
#[derive(Debug, Error)]
pub enum CheckpointError {
    /// Serialization to JSON or binary format failed
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    /// Deserialization from JSON or binary format failed
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),

    /// Checkpoint version is not supported by this version
    #[error("Unsupported checkpoint version {found}, supported: {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },

    /// Checkpoint data failed validation
    #[error("Checkpoint validation failed: {0}")]
    ValidationFailed(String),
}
