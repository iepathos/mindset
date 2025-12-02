//! Basic State Machine
//!
//! This example demonstrates a zero-cost state machine with pure transitions.
//!
//! Key concepts:
//! - Zero-cost abstractions - no runtime overhead
//! - Pure state transitions with no side effects
//! - Type-safe state representation
//! - Simple guard functions for validation
//!
//! Run with: cargo run --example basic_state_machine

use mindset::builder::{simple_transition, StateMachineBuilder};
use mindset::state_enum;

// Define connection states using the state_enum macro
state_enum! {
    enum ConnectionState {
        Disconnected,
        Connecting,
        Connected,
    }
    final: [Connected]
}

fn main() {
    println!("=== Basic State Machine Example ===\n");

    // Create a state machine with simple transitions
    let machine = StateMachineBuilder::<ConnectionState, ()>::new()
        .initial(ConnectionState::Disconnected)
        .transitions(vec![
            simple_transition(ConnectionState::Disconnected, ConnectionState::Connecting),
            simple_transition(ConnectionState::Connecting, ConnectionState::Connected),
        ])
        .build()
        .unwrap();

    println!("State machine created successfully!");
    println!("Initial state: {:?}", machine.current_state());
    println!("Is in final state: {}", machine.is_final());

    println!("\n=== Example Complete ===");
}
