use inventory_serde::prelude::*;
use inventory_serde::models::{Address, WarehouseCapacity, common::{Currency, Money}, product::{Pricing, InventoryLevels}};
use rust_decimal_macros::dec;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Inventory Management Library Demo");
    println!("=================================");

    // 1. CREATE PRODUCTS
    println!("Creating products...");
    
    let pricing = Pricing::new(dec!(125.00), dec!(199.99), "USD");
    let inventory = InventoryLevels::new(100, 10, 500, 50);
    
    let product = ProductBuilder::new()
        .sku("SKU-001".to_string())
        .name("Industrial IoT Sensor".to_string())
        .description("Advanced temperature sensor".to_string())
        .category(ProductCategory::Electronics)
        .pricing(pricing)
        .inventory(inventory)
        .build()?;
    
    println!("Created product: {} ({})", product.name, product.sku);
    println!("  Category: {:?}", product.category);
    println!("  Status: {:?}", product.status);
    println!("  Profit margin: {:.2}%", product.pricing.profit_margin_percent());

    // 2. CREATE WAREHOUSE
    println!("Creating warehouse...");
    
    let address = Address::new(
        "123 Industrial Blvd",
        "Manufacturing District", 
        "NY",
        "12345",
        "USA"
    );
    
    let capacity = WarehouseCapacity::new(dec!(10000), dec!(500000), 1000);
    
    let warehouse = WarehouseBuilder::new()
        .code("WH-001".to_string())
        .name("Main Distribution Center".to_string())
        .warehouse_type(WarehouseType::FulfillmentCenter)
        .address(address)
        .capacity(capacity)
        .build()?;
    
    println!("Created warehouse: {} ({})", warehouse.name, warehouse.code);
    println!("  Type: {:?}", warehouse.warehouse_type);
    println!("  Status: {:?}", warehouse.status);
    println!("  Address: {}, {}, {}", warehouse.address.city, warehouse.address.state, warehouse.address.country);

    // 3. CREATE SUPPLIER
    println!("Creating supplier...");
    
    let mut supplier = SupplierBuilder::new("SUP-001".to_string(), "Advanced Electronics Corp".to_string())
        .supplier_type(SupplierType::Manufacturer)
        .build();
    
    println!("Created supplier: {} ({})", supplier.name, supplier.code);
    println!("  Type: {:?}", supplier.supplier_type);
    println!("  Status: {:?}", supplier.status);

    // 4. RECORD SUPPLIER PERFORMANCE
    println!("Recording supplier performance...");
    
    let order_value = Money::new(dec!(1000.00), Currency::usd());
    
    supplier.performance.update_with_order(order_value.clone(), true, true);
    supplier.performance.update_with_order(order_value.clone(), false, true);
    supplier.performance.update_with_order(order_value, true, false);
    
    let performance = &supplier.performance;
    println!("Performance metrics:");
    println!("  Total orders: {}", performance.total_orders);
    if let Some(on_time_pct) = performance.on_time_delivery_pct {
        println!("  On-time delivery: {:.1}%", on_time_pct);
    }
    if let Some(quality_pct) = performance.quality_acceptance_pct {
        println!("  Quality acceptance: {:.1}%", quality_pct);
    }

    // 5. CREATE ORDERS
    println!("Creating orders...");
    
    let small_order = Order::builder(
        "PO-001".to_string(),
        OrderType::Purchase,
        warehouse.id,
        Currency::usd(),
    )
    .build();
    
    println!("Created purchase order: {}", small_order.order_number);
    println!("  Type: {:?}", small_order.order_type);
    println!("  Status: {:?}", small_order.status);

    // 6. CREATE STOCK TRANSACTIONS
    println!("Creating stock transactions...");
    
    let receipt_transaction = StockTransactionBuilder::new()
        .product_id(product.id)
        .source_location(warehouse.id)
        .transaction_type(TransactionType::Receipt)
        .quantity(dec!(500))
        .unit_of_measure("units".to_string())
        .build()?;
    
    println!("Created receipt transaction: {} units", receipt_transaction.quantity);
    println!("  Type: {:?}", receipt_transaction.transaction_type);
    println!("  Status: {:?}", receipt_transaction.status);

    // 7. SERIALIZATION TESTING
    println!("Testing serialization...");
    
    let product_json = serde_json::to_string_pretty(&product)?;
    println!("Serialized product to JSON ({} bytes)", product_json.len());
    
    let warehouse_json = serde_json::to_string_pretty(&warehouse)?;
    println!("Serialized warehouse to JSON ({} bytes)", warehouse_json.len());
    
    let supplier_json = serde_json::to_string_pretty(&supplier)?;
    println!("Serialized supplier to JSON ({} bytes)", supplier_json.len());

    // Test deserialization
    let deserialized_product: Product = serde_json::from_str(&product_json)?;
    println!("Deserialized product: {}", deserialized_product.name);

    println!("Demo Complete!");
    println!("Products: Created and serialized");
    println!("Warehouses: Configured with addresses");
    println!("Suppliers: Performance tracking working");
    println!("Orders: Created successfully");
    println!("Transactions: Receipt types working");
    println!("Serialization: JSON format working");
    
    Ok(())
}