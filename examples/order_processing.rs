//! E-commerce Order Processing
//!
//! This example demonstrates an order lifecycle with payment processing.
//!
//! Key concepts:
//! - E-commerce order states (Draft -> Paid -> Shipped -> Delivered)
//! - Payment integration via environment pattern
//! - Error handling in effectful actions
//! - Business validation rules
//!
//! Run with: cargo run --example order_processing

use mindset::builder::{StateMachineBuilder, TransitionBuilder};
use mindset::state_enum;

state_enum! {
    enum OrderState {
        Draft,
        Paid,
        Shipped,
        Delivered,
    }
    final: [Delivered]
}

// Order entity
struct Order {
    id: u64,
    total: f64,
    items: Vec<String>,
    shipping_address: Option<String>,
}

// Environment traits
trait PaymentGateway {
    fn process_payment(&mut self, order_id: u64, amount: f64) -> Result<String, String>;
}

trait ShippingService {
    fn create_shipment(&mut self, order_id: u64, address: &str) -> Result<String, String>;
}

trait NotificationService {
    fn notify_customer(&mut self, order_id: u64, message: &str);
}

// Pure guards
fn can_pay(order: &Order) -> bool {
    order.total > 0.0 && !order.items.is_empty()
}

fn can_ship(order: &Order) -> bool {
    order.shipping_address.is_some()
}

// Effectful actions
fn process_payment<Env>(order: &Order, env: &mut Env) -> Result<(), String>
where
    Env: PaymentGateway + NotificationService,
{
    let transaction_id = env.process_payment(order.id, order.total)?;
    env.notify_customer(
        order.id,
        &format!("Payment processed. Transaction: {}", transaction_id),
    );
    Ok(())
}

fn ship_order<Env>(order: &Order, env: &mut Env) -> Result<(), String>
where
    Env: ShippingService + NotificationService,
{
    let address = order
        .shipping_address
        .as_ref()
        .ok_or("Missing shipping address")?;
    let tracking = env.create_shipment(order.id, address)?;
    env.notify_customer(order.id, &format!("Order shipped. Tracking: {}", tracking));
    Ok(())
}

fn complete_delivery<Env>(order: &Order, env: &mut Env)
where
    Env: NotificationService,
{
    env.notify_customer(order.id, "Order delivered successfully!");
}

// Mock environment implementation
struct MockEnv {
    notifications: Vec<String>,
}

impl PaymentGateway for MockEnv {
    fn process_payment(&mut self, order_id: u64, amount: f64) -> Result<String, String> {
        println!("  [Payment] Processing ${:.2}", amount);
        Ok(format!("TXN-{}", order_id * 100))
    }
}

impl ShippingService for MockEnv {
    fn create_shipment(&mut self, order_id: u64, address: &str) -> Result<String, String> {
        println!("  [Shipping] Creating shipment to {}", address);
        Ok(format!("TRACK-{}", order_id * 1000))
    }
}

impl NotificationService for MockEnv {
    fn notify_customer(&mut self, order_id: u64, message: &str) {
        let notification = format!("Order {}: {}", order_id, message);
        println!("  [Notification] {}", notification);
        self.notifications.push(notification);
    }
}

fn main() {
    println!("=== E-commerce Order Processing ===\n");

    // Create state machine
    let _machine = StateMachineBuilder::<OrderState, ()>::new()
        .initial(OrderState::Draft)
        .add_transition(
            TransitionBuilder::new()
                .from(OrderState::Draft)
                .to(OrderState::Paid)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(OrderState::Paid)
                .to(OrderState::Shipped)
                .succeeds()
                .build()
                .unwrap(),
        )
        .add_transition(
            TransitionBuilder::new()
                .from(OrderState::Shipped)
                .to(OrderState::Delivered)
                .succeeds()
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    println!("Order processing state machine created");
    println!("States: Draft -> Paid -> Shipped -> Delivered\n");

    // Create order
    let order = Order {
        id: 12345,
        total: 149.99,
        items: vec!["Book".to_string(), "Pen".to_string()],
        shipping_address: Some("123 Main St, City, State 12345".to_string()),
    };

    let mut env = MockEnv {
        notifications: vec![],
    };

    // Process order
    println!("Processing order {}:", order.id);
    println!("  Total: ${:.2}", order.total);
    println!("  Items: {}", order.items.join(", "));
    println!();

    if can_pay(&order) {
        println!("Step 1: Process Payment");
        if let Err(e) = process_payment(&order, &mut env) {
            println!("  Error: {}\n", e);
            return;
        }
        println!();

        if can_ship(&order) {
            println!("Step 2: Ship Order");
            if let Err(e) = ship_order(&order, &mut env) {
                println!("  Error: {}\n", e);
                return;
            }
            println!();

            println!("Step 3: Complete Delivery");
            complete_delivery(&order, &mut env);
            println!();
        }
    }

    println!("Order completed successfully!");
    println!("Total notifications sent: {}", env.notifications.len());

    println!("\nKey Takeaways:");
    println!("- Models real e-commerce order lifecycle");
    println!("- Integrates multiple external services (payment, shipping)");
    println!("- Guards enforce business rules (positive total, items exist)");
    println!("- Error handling in effectful transitions");

    println!("\n=== Example Complete ===");
}
