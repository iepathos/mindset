# Checkpointing and Resume

This document describes the checkpointing and resume functionality in the MapReduce workflow system.

## Why Checkpointing Matters

Checkpointing allows long-running MapReduce workflows to be paused and resumed without losing progress. This is critical for:

- **Interruption Recovery**: Resume workflows after system crashes, network issues, or manual stops
- **Resource Management**: Pause workflows during high-demand periods and resume later
- **Cost Optimization**: Stop workflows during expensive compute times and resume during cheaper periods
- **Development/Testing**: Pause workflows to inspect intermediate state or debug issues
- **Graceful Shutdown**: Save progress before planned maintenance or upgrades

Without checkpointing, any interruption would require restarting the entire workflow from scratch, potentially wasting hours of computation.

## When to Create Checkpoints

Checkpoints are automatically created at key points during workflow execution:

1. **After Map Phase Completion**: When all map tasks finish processing
2. **After Each Reduce Round**: When a reduce iteration completes
3. **On Graceful Shutdown**: When the workflow receives a stop signal
4. **Periodic Saves**: Optionally at regular intervals during long-running phases

You can also trigger manual checkpoints by:
- Sending a `SIGUSR1` signal to the workflow process
- Calling the checkpoint API endpoint (if available)
- Using workflow control commands (implementation-specific)

## Checkpoint Structure

Checkpoints contain all state needed to resume a workflow:

```json
{
  "version": 1,
  "workflow_id": "mapreduce-2024-01-15-abc123",
  "timestamp": "2024-01-15T10:30:00Z",
  "phase": "reduce",
  "states": {
    "map_results": {
      "item_001": {"status": "completed", "output_path": "/tmp/map/item_001.json"},
      "item_002": {"status": "completed", "output_path": "/tmp/map/item_002.json"}
    },
    "reduce_results": {
      "round_1": {"status": "completed", "output_path": "/tmp/reduce/round_1.json"}
    },
    "current_round": 1
  },
  "history": [
    {"phase": "map", "started": "2024-01-15T10:00:00Z", "completed": "2024-01-15T10:20:00Z"},
    {"phase": "reduce", "round": 1, "started": "2024-01-15T10:20:00Z", "completed": "2024-01-15T10:30:00Z"}
  ],
  "metadata": {
    "total_items": 2,
    "items_processed": 2,
    "reduce_rounds_completed": 1,
    "config": {
      "max_reduce_rounds": 5,
      "convergence_threshold": 0.01
    }
  }
}
```

### Checkpoint Fields

- `version`: Checkpoint format version for backward compatibility
- `workflow_id`: Unique identifier for this workflow instance
- `timestamp`: When checkpoint was created (ISO 8601 format)
- `phase`: Current phase (map, reduce, complete, failed)
- `states`: Detailed state for each work item and phase
- `history`: Timeline of phase transitions and completions
- `metadata`: Configuration and progress metrics

## How to Restore from Checkpoint

### Basic Resume

To resume a workflow from a checkpoint:

```bash
# Resume from the default checkpoint location
./workflow resume --checkpoint ./checkpoints/latest.json

# Resume from a specific checkpoint
./workflow resume --checkpoint ./checkpoints/workflow-abc123-2024-01-15.json

# Resume with checkpoint ID (looks up in checkpoint directory)
./workflow resume --id workflow-abc123
```

### Resume Behavior

When resuming from a checkpoint:

1. **State Validation**: Verifies checkpoint integrity and version compatibility
2. **Resource Recovery**: Reattaches to existing worktrees, agents, and file handles
3. **Progress Restoration**: Skips already-completed work items
4. **Continuation**: Resumes from the exact phase and round where stopped
5. **History Preservation**: Maintains complete history including pre-checkpoint work

### Resume Example Workflow

```bash
# Start a workflow
./workflow run --config workflow.yaml

# Workflow is running... (map phase completes, reduce begins)
# User hits Ctrl+C or system interruption occurs

# Checkpoint saved automatically to ./checkpoints/workflow-abc123-latest.json

# Later, resume the workflow
./workflow resume --checkpoint ./checkpoints/workflow-abc123-latest.json

# Output shows:
# "Resuming from checkpoint created at 2024-01-15T10:30:00Z"
# "Map phase: 2/2 items completed (skipping)"
# "Reduce phase: starting round 2"
# "... workflow continues ..."
```

## Best Practices for Checkpoint Storage

### Storage Location

- **Use persistent storage**: Store checkpoints on disk, not in-memory or tmpfs
- **Separate from work directories**: Keep checkpoints independent of worktree locations
- **Version control friendly**: Use human-readable JSON format for easy inspection
- **Structured naming**: Use consistent naming like `{workflow_id}-{timestamp}.json`

### Checkpoint Management

- **Retention policy**: Keep last N checkpoints (e.g., 10) to prevent disk exhaustion
- **Cleanup on completion**: Remove checkpoints after successful workflow completion
- **Archive old checkpoints**: Move completed workflow checkpoints to archive storage
- **Backup critical checkpoints**: Copy checkpoints for long-running critical workflows

### Directory Structure Example

```
./checkpoints/
  ├── active/
  │   ├── workflow-abc123-latest.json        # Symbolic link to most recent
  │   ├── workflow-abc123-2024-01-15-1030.json
  │   └── workflow-abc123-2024-01-15-1025.json
  ├── completed/
  │   └── workflow-xyz789-2024-01-14-final.json
  └── archive/
      └── 2024-01/
          └── workflow-old123-2024-01-10-final.json
```

## Atomic Checkpoint Writes

To prevent corruption from interrupted writes, checkpoints use atomic write operations:

### Atomic Write Pattern

```rust
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Atomically writes checkpoint data to disk
///
/// Uses write-to-tempfile-then-rename pattern to ensure atomicity:
/// 1. Write to temporary file in same directory
/// 2. Flush and sync to ensure data on disk
/// 3. Rename temporary file to target path (atomic operation)
pub fn write_checkpoint_atomic(path: &Path, checkpoint_data: &[u8]) -> std::io::Result<()> {
    // Create temp file in same directory as target (same filesystem = atomic rename)
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut temp_file = NamedTempFile::new_in(dir)?;

    // Write checkpoint data
    temp_file.write_all(checkpoint_data)?;

    // Ensure data is flushed to disk before rename
    temp_file.as_file().sync_all()?;

    // Atomically replace target file with temp file
    // This operation is atomic on POSIX systems
    temp_file.persist(path)?;

    Ok(())
}

/// Example usage in checkpoint system
fn save_checkpoint(checkpoint: &Checkpoint, path: &Path) -> Result<()> {
    // Serialize checkpoint to JSON
    let json = serde_json::to_string_pretty(checkpoint)?;

    // Write atomically
    write_checkpoint_atomic(path, json.as_bytes())?;

    // Update "latest" symlink
    update_latest_symlink(path)?;

    Ok(())
}
```

### Why Atomic Writes Matter

Without atomic writes, checkpoint files can become corrupted if:
- System crashes during write
- Disk becomes full mid-write
- Process is killed while writing
- Network storage connection drops

The atomic write pattern ensures:
- **All or nothing**: File is either fully written or not written at all
- **No partial state**: Never have half-written checkpoint files
- **Immediate consistency**: No window where file is in inconsistent state
- **Safe concurrent reads**: Readers never see partial writes

### Implementation Details

The atomic write pattern works because:

1. **Temporary file**: Written completely before becoming visible
2. **Same filesystem**: Temp file must be on same filesystem as target for atomic rename
3. **Sync before rename**: `sync_all()` ensures data is on disk before rename
4. **Atomic rename**: POSIX `rename()` is atomic - old file replaced instantly with new

## Serialization Formats

The checkpoint system supports multiple serialization formats:

### JSON (Default)

- **Human-readable**: Easy to inspect and debug
- **Version-control friendly**: Text format for easy diffing
- **Slower**: Larger files and slower serialization than binary
- **Use for**: Development, debugging, small workflows

### Binary (Optional)

- **Compact**: Smaller checkpoint files
- **Fast**: Faster serialization and deserialization
- **Opaque**: Not human-readable
- **Use for**: Production, large workflows, performance-critical cases

You can configure the format in workflow settings:

```yaml
checkpoint:
  format: json  # or "binary"
  directory: ./checkpoints
  retention: 10
  auto_save_interval: 300  # seconds
```

## Troubleshooting

### Checkpoint Version Mismatch

If you see version incompatibility errors:
- Check the `version` field in checkpoint JSON
- Upgrade workflow to support older checkpoint versions
- Or migrate checkpoint to newer format using migration tool

### Corrupted Checkpoint

If checkpoint is corrupted:
- Try previous checkpoint (if retention policy kept multiple)
- Check disk space and filesystem errors
- Verify checkpoint file is complete and valid JSON
- Use `--verify` flag to validate checkpoint before resuming

### Missing Work Artifacts

If resume fails due to missing map/reduce outputs:
- Ensure work directories are preserved between runs
- Check that paths in checkpoint match actual file locations
- Use absolute paths in configuration to avoid path issues

### Performance Issues

If checkpointing slows down workflow:
- Reduce checkpoint frequency
- Use binary format instead of JSON
- Store checkpoints on faster storage
- Compress old checkpoints asynchronously
