use inventory_serde::prelude::*;
use inventory_serde::models::{Address, WarehouseCapacity, product::{Pricing, InventoryLevels}};
use rust_decimal_macros::dec;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Basic Inventory Management Demo");
    println!("==============================");

    // 1. CREATE PRODUCTS
    println!("Creating products...");
    
    let pricing = Pricing::new(dec!(50.00), dec!(75.00), "USD");
    let inventory = InventoryLevels::new(50, 5, 200, 25);
    
    let product = ProductBuilder::new()
        .sku("SKU-BASIC-001".to_string())
        .name("Basic Temperature Sensor".to_string())
        .description("Simple temperature monitoring device".to_string())
        .category(ProductCategory::Electronics)
        .pricing(pricing)
        .inventory(inventory)
        .build()?;
    
    println!("Created product: {} ({})", product.name, product.sku);
    println!("  Category: {:?}", product.category);
    println!("  Current stock: {}", product.inventory.current_stock);

    // 2. CREATE WAREHOUSE
    println!("Creating warehouse...");
    
    let address = Address::new(
        "456 Storage Ave",
        "Warehouse District",
        "CA",
        "90210",
        "USA"
    );
    
    let capacity = WarehouseCapacity::new(dec!(5000), dec!(250000), 500);
    
    let warehouse = WarehouseBuilder::new()
        .code("WH-BASIC-001".to_string())
        .name("Basic Storage Facility".to_string())
        .warehouse_type(WarehouseType::Local)
        .address(address)
        .capacity(capacity)
        .build()?;
    
    println!("Created warehouse: {} ({})", warehouse.name, warehouse.code);
    println!("  Type: {:?}", warehouse.warehouse_type);
    println!("  Total capacity: {} m³", warehouse.capacity.total_volume_m3);

    // 3. CREATE SUPPLIER
    println!("Creating supplier...");
    
    let supplier = SupplierBuilder::new("SUP-BASIC-001".to_string(), "Basic Electronics Corp".to_string())
        .supplier_type(SupplierType::Manufacturer)
        .build();
    
    println!("Created supplier: {} ({})", supplier.name, supplier.code);
    println!("  Type: {:?}", supplier.supplier_type);
    println!("  Status: {:?}", supplier.status);

    // 4. CREATE STOCK TRANSACTIONS
    println!("Creating stock transactions...");
    
    let receipt_transaction = StockTransactionBuilder::new()
        .product_id(product.id)
        .source_location(warehouse.id)
        .transaction_type(TransactionType::Receipt)
        .quantity(dec!(100))
        .unit_of_measure("units".to_string())
        .build()?;
    
    println!("Created receipt transaction: {} units", receipt_transaction.quantity);
    println!("  Type: {:?}", receipt_transaction.transaction_type);
    println!("  Status: {:?}", receipt_transaction.status);

    // 5. SERIALIZATION
    println!("Testing serialization...");
    
    let product_json = serde_json::to_string_pretty(&product)?;
    println!("Serialized product to JSON ({} bytes)", product_json.len());
    
    let warehouse_json = serde_json::to_string_pretty(&warehouse)?;
    println!("Serialized warehouse to JSON ({} bytes)", warehouse_json.len());
    
    let supplier_json = serde_json::to_string_pretty(&supplier)?;
    println!("Serialized supplier to JSON ({} bytes)", supplier_json.len());
    
    println!("Basic Demo Complete!");
    println!("====================");
    println!("Products: Created with pricing and inventory");
    println!("Warehouses: Configured with capacity");
    println!("Suppliers: Created and tracked");
    println!("Transactions: Receipt operations");
    println!("Serialization: JSON format working");
    
    Ok(())
}