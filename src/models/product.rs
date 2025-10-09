//! Product domain model for inventory management
//!
//! This module defines the core Product entity with comprehensive metadata,
//! pricing, dimensions, and inventory tracking capabilities.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Product category enumeration for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    HomeGarden,
    Sports,
    Automotive,
    Industrial,
    Healthcare,
    Food,
    Other(String),
}

impl Default for ProductCategory {
    fn default() -> Self {
        Self::Other("uncategorized".to_string())
    }
}

/// Product status in the system lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Active,
    Discontinued,
    Pending,
    OutOfStock,
    Recalled,
}

impl Default for ProductStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Physical dimensions of a product
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Dimensions {
    /// Length in centimeters
    pub length_cm: Decimal,
    /// Width in centimeters  
    pub width_cm: Decimal,
    /// Height in centimeters
    pub height_cm: Decimal,
    /// Weight in grams
    pub weight_grams: u32,
}

impl Dimensions {
    /// Create new dimensions
    pub fn new(length_cm: Decimal, width_cm: Decimal, height_cm: Decimal, weight_grams: u32) -> Self {
        Self {
            length_cm,
            width_cm,
            height_cm,
            weight_grams,
        }
    }

    /// Calculate volume in cubic centimeters
    pub fn volume_cm3(&self) -> Decimal {
        self.length_cm * self.width_cm * self.height_cm
    }

    /// Calculate shipping weight (includes packaging estimate)
    pub fn shipping_weight_grams(&self) -> u32 {
        // Add 10% for packaging
        (self.weight_grams as f64 * 1.1) as u32
    }
}

/// Product pricing information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Pricing {
    /// Cost price from supplier
    pub cost_price: Decimal,
    /// Selling price to customers
    pub selling_price: Decimal,
    /// Currency code (ISO 4217)
    pub currency: String,
    /// Minimum selling price (for discounts)
    pub min_price: Option<Decimal>,
    /// Maximum retail price
    pub max_price: Option<Decimal>,
}

impl Pricing {
    /// Create new pricing with cost and selling price
    pub fn new(cost_price: Decimal, selling_price: Decimal, currency: impl Into<String>) -> Self {
        Self {
            cost_price,
            selling_price,
            currency: currency.into(),
            min_price: None,
            max_price: None,
        }
    }

    /// Calculate profit margin percentage
    pub fn profit_margin_percent(&self) -> Decimal {
        if self.selling_price == Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        let profit = self.selling_price - self.cost_price;
        ((profit / self.selling_price) * Decimal::from(100)).round_dp(2)
    }

    /// Calculate markup percentage
    pub fn markup_percent(&self) -> Decimal {
        if self.cost_price == Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        let markup = self.selling_price - self.cost_price;
        ((markup / self.cost_price) * Decimal::from(100)).round_dp(2)
    }
}

/// Inventory levels and thresholds for stock management
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct InventoryLevels {
    /// Current stock quantity
    pub current_stock: i64,
    /// Minimum stock level (reorder point)
    pub min_stock: i64,
    /// Maximum stock level
    pub max_stock: i64,
    /// Reorder quantity
    pub reorder_quantity: i64,
    /// Reserved stock (allocated but not shipped)
    pub reserved: i64,
    /// Stock on order from suppliers
    pub on_order: i64,
}

impl InventoryLevels {
    /// Create new inventory levels
    pub fn new(current_stock: i64, min_stock: i64, max_stock: i64, reorder_quantity: i64) -> Self {
        Self {
            current_stock,
            min_stock,
            max_stock,
            reorder_quantity,
            reserved: 0,
            on_order: 0,
        }
    }

    /// Available stock (current - reserved)
    pub fn available_stock(&self) -> i64 {
        self.current_stock - self.reserved
    }

    /// Check if stock is below reorder point
    pub fn needs_reorder(&self) -> bool {
        self.available_stock() <= self.min_stock
    }

    /// Calculate stock coverage in days (requires average daily usage)
    pub fn coverage_days(&self, daily_usage: Decimal) -> Option<Decimal> {
        if daily_usage <= Decimal::ZERO {
            return None;
        }
        
        Some(Decimal::from(self.available_stock()) / daily_usage)
    }
}

/// Core Product entity for inventory management
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Product {
    /// Unique product identifier
    pub id: Uuid,
    
    /// Stock Keeping Unit (SKU) - business identifier
    pub sku: String,
    
    /// Product name
    pub name: String,
    
    /// Detailed description
    pub description: Option<String>,
    
    /// Product category
    pub category: ProductCategory,
    
    /// Current status
    pub status: ProductStatus,
    
    /// Pricing information
    pub pricing: Pricing,
    
    /// Physical dimensions
    pub dimensions: Option<Dimensions>,
    
    /// Inventory levels
    pub inventory: InventoryLevels,
    
    /// Supplier ID
    pub supplier_id: Option<Uuid>,
    
    /// Barcode (UPC/EAN)
    pub barcode: Option<String>,
    
    /// Product tags for search and categorization
    pub tags: Vec<String>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Product {
    /// Create a new product with required fields
    pub fn new(
        sku: impl Into<String>,
        name: impl Into<String>,
        pricing: Pricing,
        inventory: InventoryLevels,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            sku: sku.into(),
            name: name.into(),
            description: None,
            category: ProductCategory::default(),
            status: ProductStatus::default(),
            pricing,
            dimensions: None,
            inventory,
            supplier_id: None,
            barcode: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Update the product's updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Add a tag if it doesn't exist
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.touch();
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.touch();
            true
        } else {
            false
        }
    }

    /// Check if product is sellable
    pub fn is_sellable(&self) -> bool {
        matches!(self.status, ProductStatus::Active) && self.inventory.available_stock() > 0
    }

    /// Calculate total value of current stock
    pub fn stock_value(&self) -> Decimal {
        Decimal::from(self.inventory.current_stock) * self.pricing.cost_price
    }
}

/// Builder pattern for Product creation
#[derive(Debug, Default)]
pub struct ProductBuilder {
    sku: Option<String>,
    name: Option<String>,
    description: Option<String>,
    category: Option<ProductCategory>,
    status: Option<ProductStatus>,
    pricing: Option<Pricing>,
    dimensions: Option<Dimensions>,
    inventory: Option<InventoryLevels>,
    supplier_id: Option<Uuid>,
    barcode: Option<String>,
    tags: Vec<String>,
}

impl ProductBuilder {
    /// Create a new product builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set SKU
    pub fn sku(mut self, sku: impl Into<String>) -> Self {
        self.sku = Some(sku.into());
        self
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set category
    pub fn category(mut self, category: ProductCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set status
    pub fn status(mut self, status: ProductStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set pricing
    pub fn pricing(mut self, pricing: Pricing) -> Self {
        self.pricing = Some(pricing);
        self
    }

    /// Set dimensions
    pub fn dimensions(mut self, dimensions: Dimensions) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Set inventory levels
    pub fn inventory(mut self, inventory: InventoryLevels) -> Self {
        self.inventory = Some(inventory);
        self
    }

    /// Set supplier ID
    pub fn supplier_id(mut self, supplier_id: Uuid) -> Self {
        self.supplier_id = Some(supplier_id);
        self
    }

    /// Set barcode
    pub fn barcode(mut self, barcode: impl Into<String>) -> Self {
        self.barcode = Some(barcode.into());
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Build the product
    pub fn build(self) -> Result<Product, &'static str> {
        let sku = self.sku.ok_or("SKU is required")?;
        let name = self.name.ok_or("Name is required")?;
        let pricing = self.pricing.ok_or("Pricing is required")?;
        let inventory = self.inventory.ok_or("Inventory levels are required")?;

        let now = Utc::now();
        
        Ok(Product {
            id: Uuid::new_v4(),
            sku,
            name,
            description: self.description,
            category: self.category.unwrap_or_default(),
            status: self.status.unwrap_or_default(),
            pricing,
            dimensions: self.dimensions,
            inventory,
            supplier_id: self.supplier_id,
            barcode: self.barcode,
            tags: self.tags,
            created_at: now,
            updated_at: now,
            metadata: std::collections::HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_product_creation() {
        let pricing = Pricing::new(dec!(10.00), dec!(15.00), "USD");
        let inventory = InventoryLevels::new(100, 10, 200, 50);
        
        let product = ProductBuilder::new()
            .sku("TEST-001")
            .name("Test Product")
            .pricing(pricing)
            .inventory(inventory)
            .tag("test")
            .build()
            .unwrap();

        assert_eq!(product.sku, "TEST-001");
        assert_eq!(product.name, "Test Product");
        assert!(product.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_pricing_calculations() {
        let pricing = Pricing::new(dec!(8.00), dec!(12.00), "USD");
        
        assert_eq!(pricing.profit_margin_percent(), dec!(33.33));
        assert_eq!(pricing.markup_percent(), dec!(50.00));
    }

    #[test]
    fn test_inventory_levels() {
        let mut inventory = InventoryLevels::new(50, 10, 100, 25);
        inventory.reserved = 5;
        
        assert_eq!(inventory.available_stock(), 45);
        assert!(!inventory.needs_reorder());
        
        inventory.current_stock = 8;
        assert!(inventory.needs_reorder());
    }

    #[test]
    fn test_dimensions_calculations() {
        let dimensions = Dimensions::new(dec!(10.0), dec!(5.0), dec!(2.0), 500);
        
        assert_eq!(dimensions.volume_cm3(), dec!(100.0));
        assert_eq!(dimensions.shipping_weight_grams(), 550);
    }
}