//! Resource Management
//!
//! This example demonstrates resource lifecycle management with state machines.
//!
//! Key concepts:
//! - Resource acquisition and release patterns (RAII-like)
//! - State transitions for resource lifecycle
//! - Validation guards for resource availability
//! - Cleanup on state transitions
//!
//! Run with: cargo run --example resource_management

use mindset::builder::{StateMachineBuilder, TransitionBuilder};
use mindset::state_enum;

state_enum! {
    enum ResourceState {
        Unallocated,
        Allocated,
        Active,
        Released,
    }
    final: [Released]
}

// Resource entity
struct Resource {
    id: String,
    capacity: usize,
    in_use: usize,
}

// Environment traits
trait ResourcePool {
    fn acquire(&mut self, id: &str) -> Result<(), String>;
    fn release(&mut self, id: &str) -> Result<(), String>;
}

trait Monitor {
    fn log_usage(&mut self, resource: &Resource);
}

// Pure guards - validation logic
fn can_allocate(resource: &Resource) -> bool {
    resource.in_use == 0
}

fn can_activate(resource: &Resource) -> bool {
    resource.in_use < resource.capacity
}

fn can_release(resource: &Resource) -> bool {
    resource.in_use == 0
}

// Effectful actions
fn allocate_resource<Env>(resource: &mut Resource, env: &mut Env) -> Result<(), String>
where
    Env: ResourcePool,
{
    env.acquire(&resource.id)?;
    println!("  Allocated resource: {}", resource.id);
    Ok(())
}

fn activate_resource<Env>(resource: &mut Resource, env: &mut Env) -> Result<(), String>
where
    Env: Monitor,
{
    resource.in_use = 1;
    env.log_usage(resource);
    println!(
        "  Activated resource: {} (usage: {}/{})",
        resource.id, resource.in_use, resource.capacity
    );
    Ok(())
}

fn release_resource<Env>(resource: &mut Resource, env: &mut Env) -> Result<(), String>
where
    Env: ResourcePool,
{
    if !can_release(resource) {
        return Err(format!(
            "Cannot release resource {} with {} active users",
            resource.id, resource.in_use
        ));
    }
    env.release(&resource.id)?;
    println!("  Released resource: {}", resource.id);
    Ok(())
}

// Mock environment
struct MockEnv {
    acquired_resources: Vec<String>,
    usage_logs: Vec<String>,
}

impl ResourcePool for MockEnv {
    fn acquire(&mut self, id: &str) -> Result<(), String> {
        self.acquired_resources.push(id.to_string());
        Ok(())
    }

    fn release(&mut self, id: &str) -> Result<(), String> {
        self.acquired_resources.retain(|r| r != id);
        Ok(())
    }
}

impl Monitor for MockEnv {
    fn log_usage(&mut self, resource: &Resource) {
        let log = format!("{}: {}/{}", resource.id, resource.in_use, resource.capacity);
        self.usage_logs.push(log);
    }
}

fn main() {
    println!("=== Resource Management Example ===\n");

    // Create state machine
    let _machine = StateMachineBuilder::<ResourceState, ()>::new()
        .initial(ResourceState::Unallocated)
        .add_transition(
            TransitionBuilder::new()
                .from(ResourceState::Unallocated)
                .to(ResourceState::Allocated)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(ResourceState::Allocated)
                .to(ResourceState::Active)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(ResourceState::Active)
                .to(ResourceState::Released)
                .succeeds()
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    println!("Resource lifecycle state machine created");
    println!("States: Unallocated -> Allocated -> Active -> Released\n");

    let mut env = MockEnv {
        acquired_resources: vec![],
        usage_logs: vec![],
    };

    // Scenario 1: Normal lifecycle
    println!("Scenario 1: Normal Resource Lifecycle");
    let mut resource = Resource {
        id: "database-connection".to_string(),
        capacity: 10,
        in_use: 0,
    };

    println!("Step 1: Allocate");
    if can_allocate(&resource) {
        allocate_resource(&mut resource, &mut env).unwrap();
    }

    println!("\nStep 2: Activate");
    if can_activate(&resource) {
        activate_resource(&mut resource, &mut env).unwrap();
    }

    println!("\nStep 3: Cleanup (set in_use to 0)");
    resource.in_use = 0;
    println!("  Resource usage cleared");

    println!("\nStep 4: Release");
    if can_release(&resource) {
        release_resource(&mut resource, &mut env).unwrap();
    }
    println!();

    // Scenario 2: Try to release while still in use
    println!("Scenario 2: Prevent Release While In Use");
    let resource2 = Resource {
        id: "file-handle".to_string(),
        capacity: 1,
        in_use: 1,
    };

    if !can_release(&resource2) {
        println!(
            "  Guard failed: Resource has {} active users",
            resource2.in_use
        );
        println!("  ✗ Cannot release resource\n");
    }

    println!("Summary:");
    println!("  Acquired resources: {}", env.acquired_resources.len());
    println!("  Usage logs: {}", env.usage_logs.len());

    println!("\nKey Takeaways:");
    println!("- State machine enforces proper resource lifecycle");
    println!("- Guards prevent premature resource release");
    println!("- RAII-like pattern ensures cleanup");
    println!("- Environment pattern enables monitoring and pooling");

    println!("\n=== Example Complete ===");
}
