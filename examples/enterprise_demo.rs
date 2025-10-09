use inventory_serde::prelude::*;
use inventory_serde::models::{Address, WarehouseCapacity, common::{Currency, Money}, product::{Pricing, InventoryLevels}};
use rust_decimal_macros::dec;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Enterprise Inventory Management Demo");
    println!("====================================");

    // 1. CREATE ENTERPRISE PRODUCT CATALOG
    println!("Building enterprise product catalog...");
    
    // Premium enterprise product
    let premium_pricing = Pricing::new(dec!(750.00), dec!(1199.99), "USD");
    let premium_inventory = InventoryLevels::new(75, 15, 300, 30);
    
    let premium_product = ProductBuilder::new()
        .sku("ENT-PREM-001".to_string())
        .name("Enterprise Industrial Controller".to_string())
        .description("Advanced enterprise-grade industrial control system".to_string())
        .category(ProductCategory::Industrial)
        .pricing(premium_pricing)
        .inventory(premium_inventory)
        .build()?;
    
    // Standard business product
    let standard_pricing = Pricing::new(dec!(250.00), dec!(399.99), "USD");
    let standard_inventory = InventoryLevels::new(200, 40, 1000, 100);
    
    let standard_product = ProductBuilder::new()
        .sku("ENT-STD-001".to_string())
        .name("Business Sensor Array".to_string())
        .description("Professional multi-sensor monitoring system".to_string())
        .category(ProductCategory::Electronics)
        .pricing(standard_pricing)
        .inventory(standard_inventory)
        .build()?;
    
    println!("Created enterprise product catalog:");
    println!("  Premium: {} ({})", premium_product.name, premium_product.sku);
    println!("    Price: ${:.2} | Margin: {:.1}%", premium_product.pricing.selling_price, premium_product.pricing.profit_margin_percent());
    println!("  Standard: {} ({})", standard_product.name, standard_product.sku);
    println!("    Price: ${:.2} | Margin: {:.1}%", standard_product.pricing.selling_price, standard_product.pricing.profit_margin_percent());

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

    // 5. CREATE ENTERPRISE ORDERS
    println!("Creating enterprise procurement orders...");
    
    let enterprise_order = Order::builder(
        "ENT-PO-001".to_string(),
        OrderType::Purchase,
        warehouse.id,
        Currency::usd(),
    )
    .build();
    
    println!("Created enterprise purchase order: {}", enterprise_order.order_number);
    println!("  Type: {:?}", enterprise_order.order_type);
    println!("  Status: {:?}", enterprise_order.status);

    // 6. CREATE STOCK TRANSACTIONS
    println!("Creating stock transactions...");
    
    let receipt_transaction = StockTransactionBuilder::new()
        .product_id(premium_product.id)
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
    
    let products_json = serde_json::to_string_pretty(&vec![&premium_product, &standard_product])?;
    println!("Serialized enterprise products to JSON ({} bytes)", products_json.len());
    
    let warehouse_json = serde_json::to_string_pretty(&warehouse)?;
    println!("Serialized warehouse to JSON ({} bytes)", warehouse_json.len());
    
    let supplier_json = serde_json::to_string_pretty(&supplier)?;
    println!("Serialized supplier to JSON ({} bytes)", supplier_json.len());

    // Test deserialization
    let deserialized_products: Vec<Product> = serde_json::from_str(&products_json)?;
    println!("Deserialized {} products successfully", deserialized_products.len());

    println!("Demo Complete!");
    println!("Products: Created and serialized");
    println!("Warehouses: Configured with addresses");
    println!("Suppliers: Performance tracking working");
    println!("Orders: Created successfully");
    println!("Transactions: Receipt types working");
    println!("Serialization: JSON format working");
    
    Ok(())
}