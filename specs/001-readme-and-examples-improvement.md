---
number: 001
title: Improve README and Add Working Examples
category: documentation
priority: high
status: draft
dependencies: []
created: 2025-12-01
---

# Specification 001: Improve README and Add Working Examples

**Category**: documentation
**Priority**: high
**Status**: draft
**Dependencies**: None

## Context

The mindset project currently has a comprehensive README with inline code examples, but lacks:
1. A structured examples directory with runnable, standalone examples similar to sibling projects (stillwater, premortem, postmortem)
2. Clear documentation about how to run examples
3. Demonstration of real-world usage patterns
4. Consistent documentation structure matching the project family

The sibling projects demonstrate an excellent pattern:
- **stillwater**: 21 runnable examples covering validation, effects, parallel execution, retry patterns, resource management, etc. Each example is self-contained and demonstrates specific features.
- **premortem**: Structured examples directory with subdirectories for each example (basic/, error-demo/, layered-config/, etc.), each with its own Cargo.toml, config files, and README.md
- **postmortem**: Integration tests demonstrating JSON schema validation patterns

Mindset should follow these patterns to provide users with clear, runnable demonstrations of state machine patterns.

## Objective

Enhance the mindset project's documentation and examples to:
1. Create a comprehensive `examples/` directory with runnable examples demonstrating key features
2. Add an examples README with overview and instructions
3. Improve the main README with a clear examples section linking to runnable examples
4. Follow the established patterns from stillwater, premortem, and postmortem projects
5. Ensure all examples are tested and work correctly

## Requirements

### Functional Requirements

1. **Examples Directory Structure**
   - Create `examples/` directory at project root
   - Each example should be a standalone `.rs` file (following stillwater pattern)
   - Examples should be runnable with `cargo run --example <name>`
   - All examples must compile and run successfully

2. **Example Coverage**
   - Basic state machine (zero-cost, pure transitions)
   - Effectful state machine with environment pattern
   - Validation with enforcement rules
   - Document workflow with guards and actions
   - Traffic light (simple state transitions)
   - Order processing with payment integration
   - Account management with validation
   - Testing patterns (pure guards, mock environments)
   - Checkpoint and resume workflow
   - MapReduce workflow patterns
   - Resource management with state machines

3. **Examples README**
   - Create `examples/README.md` with overview
   - Table of runnable examples with descriptions
   - Instructions for running examples
   - Key concepts demonstrated by each example
   - Code snippets showing typical usage patterns

4. **Main README Improvements**
   - Add prominent "Examples" section after "Quick Start"
   - Table format listing examples with one-line descriptions
   - Clear instructions: `cargo run --example <name>`
   - Link to examples directory for full code
   - Ensure examples in README match runnable examples

5. **Documentation Consistency**
   - Follow similar structure to stillwater README (most comprehensive)
   - Use consistent terminology across all examples
   - Ensure code snippets are tested and accurate
   - Add doc comments explaining what each example demonstrates

### Non-Functional Requirements

1. **Quality**
   - All examples must compile without warnings
   - Examples should be self-contained and easy to understand
   - Code should follow project conventions and style
   - Each example should demonstrate a single concept clearly

2. **Maintainability**
   - Examples should use only public API
   - No hardcoded paths or environment-specific code
   - Clear comments explaining key concepts
   - Follow functional programming principles from CLAUDE.md

3. **Usability**
   - Examples should be progressively complex (simple → advanced)
   - Each example should have clear output demonstrating the feature
   - Examples should be realistic and practical (not just toy examples)

## Acceptance Criteria

- [ ] `examples/` directory exists with at least 10 runnable examples
- [ ] `examples/README.md` provides overview and instructions
- [ ] All examples compile and run without errors or warnings
- [ ] Main README has "Examples" section with table of examples
- [ ] Examples demonstrate core features:
  - [ ] Zero-cost pure state machines
  - [ ] Effectful state machines with environment pattern
  - [ ] Enforcement and validation
  - [ ] Testing patterns
  - [ ] Checkpoint and resume
  - [ ] Real-world workflows (document approval, order processing, etc.)
- [ ] Each example has clear doc comments explaining what it demonstrates
- [ ] Examples can be run with `cargo run --example <name>`
- [ ] Code follows project style and functional programming principles
- [ ] All examples are referenced in main README
- [ ] Documentation is consistent with sibling projects

## Technical Details

### Implementation Approach

1. **Phase 1: Directory Structure**
   - Create `examples/` directory
   - Add basic infrastructure (no Cargo.toml needed - examples compile as part of main project)

2. **Phase 2: Core Examples**
   - `basic_state_machine.rs` - Zero-cost state machine with pure transitions
   - `effectful_state_machine.rs` - Effects with environment pattern
   - `validation_enforcement.rs` - Enforcement rules and validation
   - `testing_patterns.rs` - Mock environments and testing strategies

3. **Phase 3: Real-World Examples**
   - `traffic_light.rs` - Simple cyclic state machine
   - `document_workflow.rs` - Multi-stage approval process
   - `order_processing.rs` - E-commerce order lifecycle
   - `account_management.rs` - Account states with validation guards

4. **Phase 4: Advanced Examples**
   - `checkpoint_resume.rs` - Long-running workflows with checkpointing
   - `mapreduce_workflow.rs` - MapReduce pattern implementation
   - `resource_management.rs` - Resource acquisition/release with state machines

5. **Phase 5: Documentation**
   - Create `examples/README.md`
   - Update main README with examples section
   - Ensure all code snippets are tested
   - Add cross-references between examples

### Example Template Structure

Each example should follow this structure:

```rust
//! [Title]
//!
//! This example demonstrates [feature/concept].
//!
//! Key concepts:
//! - [Concept 1]
//! - [Concept 2]
//!
//! Run with: cargo run --example [name]

use mindset::prelude::*;

// Type definitions
// ...

// Pure business logic functions
// ...

// Effectful functions (if needed)
// ...

fn main() {
    println!("=== [Example Title] ===\n");

    // Example implementation with clear output

    println!("\n=== Example Complete ===");
}
```

### Examples README Structure

```markdown
# Mindset Examples

This directory contains runnable examples demonstrating mindset's state machine patterns.

## Running Examples

Run any example with `cargo run --example <name>`:

```bash
cargo run --example basic_state_machine
cargo run --example document_workflow
```

## Examples Overview

| Example | Demonstrates |
|---------|--------------|
| [basic_state_machine](./basic_state_machine.rs) | Zero-cost state machine with pure transitions |
| [effectful_state_machine](./effectful_state_machine.rs) | Environment pattern and effectful actions |
| ... | ... |

## Key Concepts

### Zero-Cost Abstractions
[Explanation]

### Environment Pattern
[Explanation]

### Enforcement System
[Explanation]
```

### Main README Improvements

Add this section after "Quick Start":

```markdown
## Examples

Run any example with `cargo run --example <name>`:

| Example | Demonstrates |
|---------|--------------|
| [basic_state_machine](examples/basic_state_machine.rs) | Zero-cost state machine with pure transitions |
| [effectful_state_machine](examples/effectful_state_machine.rs) | Environment pattern and effectful actions |
| [validation_enforcement](examples/validation_enforcement.rs) | Enforcement rules and validation |
| [testing_patterns](examples/testing_patterns.rs) | Testing with mock environments |
| [traffic_light](examples/traffic_light.rs) | Simple cyclic state machine |
| [document_workflow](examples/document_workflow.rs) | Multi-stage approval workflow |
| [order_processing](examples/order_processing.rs) | E-commerce order lifecycle |
| [account_management](examples/account_management.rs) | Account states with validation |
| [checkpoint_resume](examples/checkpoint_resume.rs) | Checkpoint and resume patterns |
| [mapreduce_workflow](examples/mapreduce_workflow.rs) | MapReduce workflow implementation |
| [resource_management](examples/resource_management.rs) | Resource lifecycle management |

See [examples/](examples/) directory for full code.
```

## Dependencies

**Prerequisites**: None

**Affected Components**:
- New `examples/` directory
- New `examples/README.md`
- Existing `README.md` (additions only, no breaking changes)

**External Dependencies**: None (uses existing project dependencies)

## Testing Strategy

**Unit Tests**: Not applicable (examples are not tested via unit tests)

**Integration Tests**:
- Ensure all examples compile: `cargo build --examples`
- Ensure no warnings: `cargo clippy --examples`
- Verify examples run successfully
- Check formatting: `cargo fmt --check`

**Manual Testing**:
- Run each example individually to verify output
- Verify examples demonstrate advertised features
- Check that examples are understandable for newcomers

**User Acceptance**:
- Examples should be self-explanatory
- Output should clearly demonstrate the feature
- Code should be realistic and practical

## Documentation Requirements

**Code Documentation**:
- Each example must have module-level doc comments (`//!`)
- Key concepts should be explained in doc comments
- Complex logic should have inline comments

**User Documentation**:
- `examples/README.md` must explain all examples
- Main README must link to examples with descriptions
- Each example should reference related documentation

**Architecture Updates**:
- No ARCHITECTURE.md updates needed
- Examples demonstrate existing architecture

## Implementation Notes

### Best Practices

1. **Follow Functional Principles**
   - Pure functions for business logic (guards)
   - Effects at boundaries (actions)
   - Clear separation of concerns

2. **Use Realistic Examples**
   - Not toy examples - show real-world patterns
   - Demonstrate how to structure production code
   - Include error handling and validation

3. **Progressive Complexity**
   - Start with simple examples (basic_state_machine)
   - Build up to complex examples (mapreduce_workflow)
   - Each example should be understandable on its own

4. **Consistent Style**
   - Follow project formatting (rustfmt)
   - Use consistent naming conventions
   - Match patterns from existing documentation

### Example Categories

1. **Foundation** (basic_state_machine, effectful_state_machine)
   - Core concepts and zero-cost abstractions
   - Environment pattern and dependency injection

2. **Validation** (validation_enforcement, testing_patterns)
   - Enforcement rules and guards
   - Testing strategies and mock environments

3. **Real-World** (traffic_light, document_workflow, order_processing, account_management)
   - Practical state machine patterns
   - Domain-specific examples

4. **Advanced** (checkpoint_resume, mapreduce_workflow, resource_management)
   - Complex workflows and patterns
   - Performance and resilience features

### References to Sibling Projects

**Stillwater Pattern**:
- Each example is a standalone `.rs` file in `examples/`
- Examples use `//!` doc comments for module-level documentation
- Clear structure: imports → types → logic → main
- Examples demonstrate specific features in isolation

**Premortem Pattern**:
- Subdirectory structure for complex examples (optional for mindset)
- Clear README with running instructions
- Examples table showing what each demonstrates

**Common Patterns**:
- "Run with: cargo run --example <name>" in doc comments
- Clear output showing what the example demonstrates
- Realistic code that could be adapted for production use

## Migration and Compatibility

**Breaking Changes**: None - this is purely additive

**Migration Requirements**: None

**Compatibility Considerations**:
- Examples must work with current public API
- Should not expose internal implementation details
- Examples demonstrate recommended patterns

## Success Metrics

1. **Completeness**: At least 10 working examples covering core features
2. **Quality**: All examples compile without warnings, follow project conventions
3. **Clarity**: Examples are self-explanatory and demonstrate clear concepts
4. **Consistency**: Documentation matches code, follows sibling project patterns
5. **Discoverability**: Easy to find and run examples from README

## Future Enhancements

1. **Interactive Examples**: Web-based playground (future work)
2. **Video Tutorials**: Screencasts walking through examples (future work)
3. **Example Tests**: Integration tests that run examples as tests (future work)
4. **Benchmarks**: Performance examples comparing pure vs effectful (future work)
5. **Advanced Patterns**: More complex real-world scenarios (iterative additions)
