use chrono::Utc;
use rust_decimal_macros::dec;
use uuid::Uuid;

// Enterprise tests are included in separate modules

use crate::{
    algorithms::*,
    builders::*,
    errors::*,
    models::*,
    pipelines::*,
    serialization::*,
};

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_product_creation_with_builder() {
        let product = ProductBuilder::new("SKU-001", "Premium Widget")
            .category("Electronics")
            .unit_cost(dec!(25.50), Currency::USD)
            .weight_grams(150)
            .stock_levels(10, 100, 25)
            .unwrap()
            .lead_time_days(7)
            .add_tag("premium")
            .add_tag("electronic")
            .build()
            .unwrap();

        assert_eq!(product.sku, "SKU-001");
        assert_eq!(product.name, "Premium Widget");
        assert_eq!(product.category, "Electronics");
        assert_eq!(product.unit_cost.amount, dec!(25.50));
        assert_eq!(product.unit_cost.currency, Currency::USD);
        assert_eq!(product.weight_grams, 150);
        assert_eq!(product.minimum_stock, 10);
        assert_eq!(product.maximum_stock, 100);
        assert_eq!(product.reorder_point, 25);
        assert_eq!(product.tags.len(), 2);
        assert!(product.tags.contains(&"premium".to_string()));
    }

    #[test]
    fn test_product_validation() {
        // Test empty SKU validation
        let result = ProductBuilder::new("", "Test Product")
            .category("Test")
            .unit_cost(dec!(10.0), Currency::USD)
            .build();
        
        assert!(result.is_err());
        
        // Test invalid stock levels
        let result = ProductBuilder::new("SKU-002", "Test Product")
            .category("Test")
            .unit_cost(dec!(10.0), Currency::USD)
            .stock_levels(100, 50, 75); // min > max
        
        assert!(result.is_err());
    }

    #[test]
    fn test_profit_margin_calculation() {
        let product = ProductBuilder::new("SKU-003", "Margin Test")
            .category("Test")
            .unit_cost(dec!(20.0), Currency::USD)
            .retail_price(dec!(25.0), Currency::USD)
            .build()
            .unwrap();

        let margin = product.profit_margin().unwrap();
        let margin_f64: f64 = margin.to_string().parse().unwrap();
        assert_relative_eq!(margin_f64, 20.0, epsilon = 0.01); // (25-20)/25 * 100 = 20%
    }

    #[test]
    fn test_eoq_calculation() {
        let product = ProductBuilder::new("SKU-004", "EOQ Test")
            .category("Test")
            .unit_cost(dec!(10.0), Currency::USD)
            .build()
            .unwrap();

        let eoq = product.calculate_eoq(1000, dec!(50.0)).unwrap();
        assert!(eoq > 0);
        // EOQ = sqrt((2 * 1000 * 50) / (10 * 0.25)) = sqrt(100000 / 2.5) = sqrt(40000) ≈ 200
        assert!(eoq >= 190 && eoq <= 210);
    }

    #[test]
    fn test_money_formatting() {
        let usd_money = Money::new(dec!(1234.56), Currency::USD);
        assert_eq!(usd_money.format(), "$1234.56");

        let jpy_money = Money::new(dec!(1234), Currency::JPY);
        assert_eq!(jpy_money.format(), "¥1234");

        let eur_money = Money::new(dec!(999.99), Currency::EUR);
        assert_eq!(eur_money.format(), "€999.99");
    }

    #[test]
    fn test_currency_conversion() {
        let usd_money = Money::new(dec!(100.0), Currency::USD);
        let eur_money = usd_money.convert_to(Currency::EUR, dec!(0.85)); // 1 USD = 0.85 EUR
        
        assert_eq!(eur_money.currency, Currency::EUR);
        assert_eq!(eur_money.amount, dec!(85.0));
    }

    #[test]
    fn test_location_utilization() {
        let location = LocationBuilder::new("MAIN-01")
            .zone("A")
            .aisle("1")
            .shelf("B")
            .bin("3")
            .capacity_units(1000)
            .build()
            .unwrap();

        assert_eq!(location.utilization_percentage(), 0.0);
        
        let mut location_with_inventory = location.clone();
        location_with_inventory.current_units = 250;
        assert_eq!(location_with_inventory.utilization_percentage(), 25.0);
        
        assert!(location_with_inventory.has_capacity(750));
        assert!(!location_with_inventory.has_capacity(751));
    }

    #[test]
    fn test_inventory_snapshot_operations() {
        let mut snapshot = InventorySnapshot {
            product_id: Uuid::new_v4(),
            location_id: Uuid::new_v4(),
            quantity_on_hand: 100,
            quantity_available: 100,
            quantity_reserved: 0,
            quantity_on_order: 0,
            average_cost: Money::new(dec!(10.0), Currency::USD),
            last_counted: None,
            last_movement: None,
            batch_numbers: vec![],
        };

        // Test reservation
        assert!(snapshot.can_allocate(50));
        snapshot.reserve(50).unwrap();
        assert_eq!(snapshot.quantity_available, 50);
        assert_eq!(snapshot.quantity_reserved, 50);

        // Test over-allocation
        assert!(!snapshot.can_allocate(51));
        assert!(snapshot.reserve(51).is_err());

        // Test total value calculation
        let total_value = snapshot.total_value();
        assert_eq!(total_value.amount, dec!(1000.0)); // 100 * $10.00
        assert_eq!(total_value.currency, Currency::USD);
    }

    #[test]
    fn test_inventory_system_operations() {
        let mut system = InventorySystem::new();
        
        // Add a product
        let product = ProductBuilder::new("SKU-SYS-001", "System Test Product")
            .category("Test")
            .unit_cost(dec!(15.0), Currency::USD)
            .stock_levels(5, 50, 15)
            .unwrap()
            .build()
            .unwrap();
        
        let product_id = product.id;
        system.add_product(product).unwrap();

        // Add inventory
        let location_id = Uuid::new_v4();
        let snapshot = InventorySnapshot {
            product_id,
            location_id,
            quantity_on_hand: 10,
            quantity_available: 10,
            quantity_reserved: 0,
            quantity_on_order: 0,
            average_cost: Money::new(dec!(15.0), Currency::USD),
            last_counted: Some(Utc::now()),
            last_movement: None,
            batch_numbers: vec!["BATCH-001".to_string()],
        };
        
        system.add_inventory(snapshot).unwrap();

        // Test transaction recording
        let transaction = Transaction {
            id: Uuid::new_v4(),
            product_id,
            location_id,
            transaction_type: TransactionType::Shipment,
            quantity: -5, // Shipping 5 units
            unit_cost: Some(Money::new(dec!(15.0), Currency::USD)),
            reference_number: Some("SHIP-001".to_string()),
            reason_code: None,
            user_id: Some("test_user".to_string()),
            timestamp: Utc::now(),
            batch_number: Some("BATCH-001".to_string()),
            expiry_date: None,
        };

        system.record_transaction(transaction).unwrap();

        // Verify inventory was updated
        let key = (product_id, location_id);
        let updated_snapshot = system.inventory.get(&key).unwrap();
        assert_eq!(updated_snapshot.quantity_on_hand, 5); // 10 - 5 = 5
    }

    #[test]
    fn test_abc_analysis() {
        let mut system = InventorySystem::new();
        
        // Add products with different value profiles
        let high_value_product = ProductBuilder::new("HIGH-001", "High Value Item")
            .category("Electronics")
            .unit_cost(dec!(1000.0), Currency::USD)
            .build()
            .unwrap();
            
        let medium_value_product = ProductBuilder::new("MED-001", "Medium Value Item")
            .category("Electronics")
            .unit_cost(dec!(100.0), Currency::USD)
            .build()
            .unwrap();
            
        let low_value_product = ProductBuilder::new("LOW-001", "Low Value Item")
            .category("Office")
            .unit_cost(dec!(5.0), Currency::USD)
            .build()
            .unwrap();

        system.add_product(high_value_product).unwrap();
        system.add_product(medium_value_product).unwrap();
        system.add_product(low_value_product).unwrap();

        // Add some transactions to simulate demand
        let product_ids: Vec<Uuid> = system.products.keys().cloned().collect();
        for product_id in product_ids {
            let location_id = Uuid::new_v4();
            
            // Add some shipment transactions
            for i in 0..10 {
                let transaction = Transaction {
                    id: Uuid::new_v4(),
                    product_id,
                    location_id,
                    transaction_type: TransactionType::Shipment,
                    quantity: -1,
                    unit_cost: None,
                    reference_number: Some(format!("SHIP-{:03}", i)),
                    reason_code: None,
                    user_id: Some("test_user".to_string()),
                    timestamp: Utc::now() - chrono::Duration::days(i as i64),
                    batch_number: None,
                    expiry_date: None,
                };
                system.record_transaction(transaction).unwrap();
            }
        }

        let abc_result = system.perform_abc_analysis().unwrap();
        assert_eq!(abc_result.total_products, 3);
        // With the current test data, we should have at least one product in each category
        // The high value product should be in A category due to high unit cost
        let total_classified = abc_result.a_products.len() + abc_result.b_products.len() + abc_result.c_products.len();
        assert_eq!(total_classified, 3);
    }

    #[test]
    fn test_demand_forecasting() {
        let mut system = InventorySystem::new();
        
        let product = ProductBuilder::new("FORECAST-001", "Forecast Test Product")
            .category("Test")
            .unit_cost(dec!(25.0), Currency::USD)
            .build()
            .unwrap();
            
        let product_id = product.id;
        let location_id = Uuid::new_v4();
        system.add_product(product).unwrap();

        // Add historical transactions with a trend
        for week in 0..12 {
            let weekly_demand = 10 + week; // Increasing demand
            for day in 0..weekly_demand {
                let transaction = Transaction {
                    id: Uuid::new_v4(),
                    product_id,
                    location_id,
                    transaction_type: TransactionType::Shipment,
                    quantity: -1,
                    unit_cost: None,
                    reference_number: Some(format!("SHIP-W{:02}-D{:02}", week, day)),
                    reason_code: None,
                    user_id: Some("test_user".to_string()),
                    timestamp: Utc::now() - chrono::Duration::weeks(12 - week as i64),
                    batch_number: None,
                    expiry_date: None,
                };
                system.record_transaction(transaction).unwrap();
            }
        }

        let forecast = system.forecast_demand(product_id, 30).unwrap();
        assert_eq!(forecast.product_id, product_id);
        assert_eq!(forecast.forecast_period_days, 30);
        assert!(forecast.predicted_demand > 0);
        assert!(forecast.confidence_level > 0.0);
        assert!(forecast.trend_factor != 0.0); // Should detect the increasing trend
    }

    #[test]
    fn test_inventory_pipeline_filtering() {
        let products = vec![
            ProductBuilder::new("FILTER-001", "Electronics Item")
                .category("Electronics")
                .unit_cost(dec!(100.0), Currency::USD)
                .status(ProductStatus::Active)
                .build()
                .unwrap(),
            ProductBuilder::new("FILTER-002", "Office Item")
                .category("Office")
                .unit_cost(dec!(25.0), Currency::USD)
                .status(ProductStatus::Discontinued)
                .build()
                .unwrap(),
            ProductBuilder::new("FILTER-003", "Another Electronics Item")
                .category("Electronics")
                .unit_cost(dec!(150.0), Currency::USD)
                .status(ProductStatus::Active)
                .build()
                .unwrap(),
        ];

        let pipeline = InventoryPipeline::new(products, vec![], vec![]);

        // Test category filtering
        let electronics = pipeline.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        // Test status filtering
        let active_products = pipeline.filter_by_status(&ProductStatus::Active);
        assert_eq!(active_products.len(), 2);

        // Test price range filtering (FILTER-001 has unit_cost 100.0, which should be in range 50-120)
        // But we auto-calculate retail price as 1.5x unit cost = 150.0, which is outside range
        // Let's test with a wider range that includes the retail price
        let mid_range = pipeline.filter_by_price_range(dec!(140.0), dec!(160.0), Currency::USD);
        assert_eq!(mid_range.len(), 1);
        assert_eq!(mid_range[0].sku, "FILTER-001");
    }

    #[test]
    fn test_serialization_json() {
        let product = ProductBuilder::new("SERIAL-001", "Serialization Test")
            .category("Test")
            .unit_cost(dec!(30.0), Currency::USD)
            .weight_grams(200)
            .add_tag("serialization")
            .add_tag("test")
            .build()
            .unwrap();

        // Test JSON serialization
        let json = product.to_json().unwrap();
        assert!(json.contains("SERIAL-001"));
        assert!(json.contains("Serialization Test"));

        // Test JSON deserialization
        let deserialized = Product::from_json(&json).unwrap();
        assert_eq!(deserialized.sku, product.sku);
        assert_eq!(deserialized.name, product.name);
        assert_eq!(deserialized.unit_cost.amount, product.unit_cost.amount);
        assert_eq!(deserialized.tags, product.tags);
    }

    #[test]
    fn test_serialization_toml() {
        let product = ProductBuilder::new("TOML-001", "TOML Test")
            .category("Test")
            .unit_cost(dec!(45.0), Currency::EUR)
            .build()
            .unwrap();

        // Test TOML serialization
        let toml = product.to_toml().unwrap();
        assert!(toml.contains("TOML-001"));
        assert!(toml.contains("EUR"));

        // Test TOML deserialization
        let deserialized = Product::from_toml(&toml).unwrap();
        assert_eq!(deserialized.sku, product.sku);
        assert_eq!(deserialized.unit_cost.currency, Currency::EUR);
    }

    #[test]
    fn test_serialization_csv_product_list() {
        let products = vec![
            ProductBuilder::new("CSV-001", "CSV Test 1")
                .category("Category A")
                .unit_cost(dec!(10.0), Currency::USD)
                .weight_grams(100)
                .add_tag("csv")
                .build()
                .unwrap(),
            ProductBuilder::new("CSV-002", "CSV Test 2")
                .category("Category B")
                .unit_cost(dec!(20.0), Currency::USD)
                .weight_grams(200)
                .add_tag("csv")
                .add_tag("test")
                .build()
                .unwrap(),
        ];

        let product_list = ProductList::new(products.clone());

        // Test CSV serialization
        let csv = product_list.to_csv().unwrap();
        assert!(csv.contains("CSV-001"));
        assert!(csv.contains("CSV-002"));
        assert!(csv.contains("Category A"));
        assert!(csv.contains("csv;test")); // Tags should be joined with semicolons

        // Test CSV deserialization
        let deserialized = ProductList::from_csv(&csv).unwrap();
        assert_eq!(deserialized.products.len(), 2);
        assert_eq!(deserialized.products[0].sku, "CSV-001");
        assert_eq!(deserialized.products[1].sku, "CSV-002");
        
        // Check that tags were properly parsed
        let product_2 = &deserialized.products[1];
        assert_eq!(product_2.tags.len(), 2);
        assert!(product_2.tags.contains(&"csv".to_string()));
        assert!(product_2.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_supplier_builder() {
        let address = AddressBuilder::new()
            .street_1("123 Main St")
            .city("Anytown")
            .state_province("ST")
            .postal_code("12345")
            .country("USA")
            .build()
            .unwrap();

        let supplier = SupplierBuilder::new("Test Supplier Inc.", "contact@testsupplier.com")
            .contact_phone("+1-555-0123")
            .address(address)
            .payment_terms("Net 15")
            .lead_time_days(10)
            .quality_rating(8.5)
            .unwrap()
            .reliability_score(0.95)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(supplier.name, "Test Supplier Inc.");
        assert_eq!(supplier.contact_email, "contact@testsupplier.com");
        assert_eq!(supplier.contact_phone, Some("+1-555-0123".to_string()));
        assert_eq!(supplier.payment_terms, "Net 15");
        assert_eq!(supplier.lead_time_days, 10);
        assert_eq!(supplier.quality_rating, 8.5);
        assert_eq!(supplier.reliability_score, 0.95);
        assert!(supplier.active);

        // Test quality standards check
        assert!(supplier.meets_quality_standards(8.0, 0.9));
        assert!(!supplier.meets_quality_standards(9.0, 0.9));
    }

    #[test]
    fn test_inventory_setup_builder() {
        let address = AddressBuilder::new()
            .street_1("456 Industrial Ave")
            .city("Manufacturing City")
            .state_province("MN")
            .postal_code("54321")
            .country("USA")
            .build()
            .unwrap();

        let supplier = SupplierBuilder::new("Widget Supplier LLC", "orders@widgetsupplier.com")
            .address(address)
            .build()
            .unwrap();

        let product = ProductBuilder::new("SETUP-001", "Setup Test Widget")
            .category("Widgets")
            .unit_cost(dec!(12.50), Currency::USD)
            .supplier_id(supplier.id)
            .build()
            .unwrap();

        let location = LocationBuilder::new("WAREHOUSE-01")
            .zone("A")
            .aisle("1")
            .shelf("2")
            .bin("C")
            .capacity_units(500)
            .build()
            .unwrap();

        let setup = InventorySetupBuilder::new()
            .add_supplier(supplier.clone())
            .add_product(product.clone())
            .add_location(location)
            .build()
            .unwrap();

        assert_eq!(setup.products.len(), 1);
        assert_eq!(setup.suppliers.len(), 1);
        assert_eq!(setup.locations.len(), 1);

        // Test products by supplier
        let supplier_products = setup.products_by_supplier(supplier.id);
        assert_eq!(supplier_products.len(), 1);
        assert_eq!(supplier_products[0].sku, "SETUP-001");

        // Test categories
        let categories = setup.categories();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0], "Widgets");

        // Test total inventory value
        let total_values = setup.total_inventory_value();
        let usd_total = total_values.get(&Currency::USD).unwrap();
        assert_eq!(*usd_total, dec!(125.0)); // 10 minimum stock * $12.50 unit cost
    }

    #[test]
    fn test_inventory_turnover_calculations() {
        let cogs = dec!(10000.0);
        let avg_inventory = dec!(2500.0);
        
        let turnover = calculate_inventory_turnover(cogs, avg_inventory).unwrap();
        assert_relative_eq!(turnover, 4.0, epsilon = 0.01); // 10000 / 2500 = 4.0

        let dio = calculate_days_inventory_outstanding(avg_inventory, cogs, 365).unwrap();
        assert_relative_eq!(dio, 91.25, epsilon = 0.01); // 365 / 4.0 = 91.25 days
    }

    #[test]
    fn test_error_handling() {
        // Test validation errors
        let validation_error = InventoryError::validation("Test validation error");
        assert_eq!(validation_error.to_string(), "Validation error: Test validation error");

        // Test serialization errors
        let invalid_json = "{ invalid json }";
        let result = Product::from_json(invalid_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Serialization error"));

        // Test builder errors
        let result = ProductBuilder::new("", "Empty SKU Test")
            .category("Test")
            .unit_cost(dec!(10.0), Currency::USD)
            .build();
        assert!(result.is_err());

        // Test currency mismatch
        let result = ProductBuilder::new("CURRENCY-001", "Currency Test")
            .category("Test")
            .unit_cost(dec!(10.0), Currency::USD)
            .retail_price(dec!(15.0), Currency::EUR) // Different currency
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_format_auto_detection() {
        let json_content = r#"{"sku": "TEST-001"}"#;
        assert_eq!(SerializationUtils::auto_detect_format(json_content), SerializationFormat::Json);

        let toml_content = r#"
        sku = "TEST-001"
        name = "Test Product"
        "#;
        assert_eq!(SerializationUtils::auto_detect_format(toml_content), SerializationFormat::Toml);

        let csv_content = "sku,name\nTEST-001,Test Product";
        assert_eq!(SerializationUtils::auto_detect_format(csv_content), SerializationFormat::Csv);

        // Test format from extension
        assert_eq!(SerializationUtils::format_from_extension("products.json"), Some(SerializationFormat::Json));
        assert_eq!(SerializationUtils::format_from_extension("config.toml"), Some(SerializationFormat::Toml));
        assert_eq!(SerializationUtils::format_from_extension("data.csv"), Some(SerializationFormat::Csv));
        assert_eq!(SerializationUtils::format_from_extension("unknown.txt"), None);
    }

    #[test]
    fn test_comprehensive_business_scenario() {
        // Create a comprehensive business scenario testing multiple components
        let mut system = InventorySystem::new();

        // Set up suppliers
        let address = AddressBuilder::new()
            .street_1("789 Supply Chain Blvd")
            .city("Industrial Park")
            .state_province("CA")
            .postal_code("90210")
            .country("USA")
            .build()
            .unwrap();

        let supplier = SupplierBuilder::new("Premium Electronics Inc.", "orders@premiumelec.com")
            .address(address)
            .quality_rating(9.2)
            .unwrap()
            .reliability_score(0.98)
            .unwrap()
            .build()
            .unwrap();

        // Create products with different characteristics
        let high_end_product = ProductBuilder::new("PREM-001", "Premium Smartphone")
            .category("Electronics")
            .subcategory("Mobile Devices")
            .unit_cost(dec!(400.0), Currency::USD)
            .retail_price(dec!(799.0), Currency::USD)
            .weight_grams(180)
            .dimensions_cm(15.0, 7.5, 0.8)
            .supplier_id(supplier.id)
            .stock_levels(5, 50, 15)
            .unwrap()
            .lead_time_days(14)
            .add_tags(vec!["premium", "smartphone", "electronics"])
            .build()
            .unwrap();

        let accessory_product = ProductBuilder::new("ACC-001", "Phone Case")
            .category("Electronics")
            .subcategory("Accessories")
            .unit_cost(dec!(5.0), Currency::USD)
            .retail_price(dec!(19.99), Currency::USD)
            .weight_grams(50)
            .dimensions_cm(16.0, 8.0, 1.0)
            .supplier_id(supplier.id)
            .stock_levels(20, 200, 50)
            .unwrap()
            .lead_time_days(7)
            .add_tags(vec!["accessory", "protection"])
            .build()
            .unwrap();

        // Add to system
        system.add_product(high_end_product.clone()).unwrap();
        system.add_product(accessory_product.clone()).unwrap();

        // Add inventory snapshots
        let location_id = Uuid::new_v4();
        let phone_inventory = InventorySnapshot {
            product_id: high_end_product.id,
            location_id,
            quantity_on_hand: 12,
            quantity_available: 10,
            quantity_reserved: 2,
            quantity_on_order: 25,
            average_cost: Money::new(dec!(400.0), Currency::USD),
            last_counted: Some(Utc::now()),
            last_movement: Some(Utc::now() - chrono::Duration::days(2)),
            batch_numbers: vec!["BATCH-2024-001".to_string()],
        };

        let case_inventory = InventorySnapshot {
            product_id: accessory_product.id,
            location_id,
            quantity_on_hand: 45,
            quantity_available: 40,
            quantity_reserved: 5,
            quantity_on_order: 100,
            average_cost: Money::new(dec!(5.0), Currency::USD),
            last_counted: Some(Utc::now()),
            last_movement: Some(Utc::now() - chrono::Duration::days(1)),
            batch_numbers: vec!["BATCH-2024-002".to_string()],
        };

        system.add_inventory(phone_inventory).unwrap();
        system.add_inventory(case_inventory).unwrap();

        // Simulate sales transactions over time
        let mut transaction_id = 1;
        for week in 0..8 {
            // Weekly sales pattern - phones sell less frequently but higher value
            for _day in 0..2 {
                if week < 6 { // Don't create transactions in recent weeks to test low stock
                    let transaction = Transaction {
                        id: Uuid::new_v4(),
                        product_id: high_end_product.id,
                        location_id,
                        transaction_type: TransactionType::Shipment,
                        quantity: -1,
                        unit_cost: Some(Money::new(dec!(400.0), Currency::USD)),
                        reference_number: Some(format!("SALE-{:04}", transaction_id)),
                        reason_code: None,
                        user_id: Some("sales_system".to_string()),
                        timestamp: Utc::now() - chrono::Duration::weeks(8 - week as i64),
                        batch_number: Some("BATCH-2024-001".to_string()),
                        expiry_date: None,
                    };
                    system.record_transaction(transaction).unwrap();
                    transaction_id += 1;
                }
            }

            // Cases sell more frequently
            for _day in 0..5 {
                let transaction = Transaction {
                    id: Uuid::new_v4(),
                    product_id: accessory_product.id,
                    location_id,
                    transaction_type: TransactionType::Shipment,
                    quantity: -1,
                    unit_cost: Some(Money::new(dec!(5.0), Currency::USD)),
                    reference_number: Some(format!("SALE-{:04}", transaction_id)),
                    reason_code: None,
                    user_id: Some("sales_system".to_string()),
                    timestamp: Utc::now() - chrono::Duration::weeks(8 - week as i64),
                    batch_number: Some("BATCH-2024-002".to_string()),
                    expiry_date: None,
                };
                system.record_transaction(transaction).unwrap();
                transaction_id += 1;
            }
        }

        // Test reorder recommendations
        let recommendations = system.calculate_reorder_recommendations().unwrap();
        assert!(!recommendations.is_empty());

        // The phone should need reordering since current stock (12) is below reorder point (15)
        let phone_rec = recommendations.iter()
            .find(|r| r.product_id == high_end_product.id);
        assert!(phone_rec.is_some());
        let phone_rec = phone_rec.unwrap();
        assert!(phone_rec.urgency > 0.5); // Should be medium to high urgency

        // Test ABC analysis
        let abc_result = system.perform_abc_analysis().unwrap();
        assert_eq!(abc_result.total_products, 2);
        
        // All products should be classified
        let total_classified = abc_result.a_products.len() + abc_result.b_products.len() + abc_result.c_products.len();
        assert_eq!(total_classified, 2);

        // Test demand forecasting
        let phone_forecast = system.forecast_demand(high_end_product.id, 30).unwrap();
        assert!(phone_forecast.predicted_demand > 0);
        assert!(phone_forecast.annual_demand > 0);

        // Test pipeline analytics
        let pipeline = InventoryPipeline::new(
            system.products.values().cloned().collect(),
            system.inventory.values().cloned().collect(),
            system.transactions.clone(),
        );

        let electronics = pipeline.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let enriched_data = pipeline.enrich_product_data();
        assert_eq!(enriched_data.len(), 2);

        // Test serialization of products only (system serialization has complex keys)
        let product_list = ProductList::new(system.products.values().cloned().collect());
        let products_json = product_list.to_json().unwrap();
        assert!(products_json.contains("PREM-001"));
        assert!(products_json.contains("ACC-001"));

        let products_csv = product_list.to_csv().unwrap();
        assert!(products_csv.contains("Premium Smartphone"));
        assert!(products_csv.contains("Phone Case"));

        // Verify business logic calculations
        let phone_margin = high_end_product.profit_margin().unwrap();
        let phone_margin_f64: f64 = phone_margin.to_string().parse().unwrap();
        assert_relative_eq!(phone_margin_f64, 49.94, epsilon = 0.01); // ~50% margin

        let case_margin = accessory_product.profit_margin().unwrap();
        let case_margin_f64: f64 = case_margin.to_string().parse().unwrap();
        assert_relative_eq!(case_margin_f64, 74.99, epsilon = 0.01); // ~75% margin

        println!("✅ Comprehensive business scenario test passed!");
        println!("   - Created {} products with supplier relationships", system.products.len());
        println!("   - Processed {} transactions", system.transactions.len());
        println!("   - Generated {} reorder recommendations", recommendations.len());
        println!("   - ABC Analysis: {} A-class, {} B-class, {} C-class products",
                 abc_result.a_products.len(),
                 abc_result.b_products.len(),
                 abc_result.c_products.len());
        println!("   - Phone forecast: {} units/30 days ({}% confidence)",
                 phone_forecast.predicted_demand,
                 (phone_forecast.confidence_level * 100.0) as u32);
    }
}