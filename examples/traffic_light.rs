//! Traffic Light State Machine
//!
//! This example demonstrates a simple cyclic state machine.
//!
//! Key concepts:
//! - Cyclic state transitions (states repeat)
//! - Simple state enumeration
//! - Zero-cost pure transitions
//! - Practical real-world pattern
//!
//! Run with: cargo run --example traffic_light

use mindset::builder::{simple_transition, StateMachineBuilder};
use mindset::state_enum;

state_enum! {
    enum TrafficLight {
        Red,
        Yellow,
        Green,
    }
}

fn main() {
    println!("=== Traffic Light State Machine ===\n");

    // Create cyclic state machine
    let machine = StateMachineBuilder::<TrafficLight, ()>::new()
        .initial(TrafficLight::Red)
        .transitions(vec![
            simple_transition(TrafficLight::Red, TrafficLight::Green),
            simple_transition(TrafficLight::Green, TrafficLight::Yellow),
            simple_transition(TrafficLight::Yellow, TrafficLight::Red),
        ])
        .build()
        .unwrap();

    println!("Traffic light state machine created");
    println!("Initial state: {:?}\n", machine.current_state());

    println!("Transition sequence:");
    println!("  Red -> Green    (Go!)");
    println!("  Green -> Yellow (Caution)");
    println!("  Yellow -> Red   (Stop)\n");

    println!("This is a cyclic state machine - the sequence repeats:");
    println!("  Red -> Green -> Yellow -> Red -> Green -> ...\n");

    println!("Key Characteristics:");
    println!("- Zero-cost transitions (compiles to simple state updates)");
    println!("- Type-safe state enumeration");
    println!("- No final state (cycles indefinitely)");
    println!("- Models real-world traffic control");

    println!("\n=== Example Complete ===");
}
