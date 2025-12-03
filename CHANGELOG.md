# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-12-03

### Added

#### Core State Machine (Spec 001)
- Pure functional state machine implementation with zero-cost abstractions
- Type-safe state transitions with compile-time guarantees
- Pure guard functions for deterministic state validation
- State trait for defining custom state types
- Transition history tracking
- Support for final states

#### Effect System (Spec 002)
- Effect-based transitions using Stillwater 0.11.0
- Environment pattern for clean dependency injection
- Trait-based environment composition
- Zero-cost pure transitions when effects aren't needed
- Explicit effectful transitions with environment parameters
- Clear separation between pure guards and effectful actions

#### Checkpoint and Resume (Spec 004)
- Automatic checkpointing for long-running workflows
- Support for JSON and binary serialization formats
- Atomic checkpoint writes to prevent corruption
- Resume workflows from saved checkpoints
- State preservation across interruptions
- MapReduce workflow support with checkpoint integration

#### Builder API (Spec 005)
- Ergonomic `StateMachineBuilder` for machine construction
- `TransitionBuilder` for defining transitions
- `state_enum!` macro for deriving State trait
- Helper functions: `simple_transition`, `conditional_transition`
- Comprehensive error handling with descriptive messages
- Type-safe builder pattern with compile-time validation

#### Documentation
- Comprehensive README with quick start guide
- Builder guide with examples and patterns
- Checkpointing guide with best practices
- Effects guide for environment pattern usage
- 10+ working examples covering common use cases:
  - Basic state machine
  - Effectful state machine
  - Traffic light
  - Document workflow
  - Order processing
  - Account management
  - Checkpoint resume
  - MapReduce workflow
  - Resource management
  - Testing patterns

#### Project Infrastructure
- MIT license
- GitHub Actions CI/CD workflows
- Cargo deny configuration for dependency auditing
- Rust toolchain specification
- Property-based testing with proptest
- Comprehensive unit and integration tests

### Changed
- Removed enforcement system from core library to maintain focus on state machine primitives

[Unreleased]: https://github.com/iepathos/mindset/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/iepathos/mindset/releases/tag/v0.1.0
