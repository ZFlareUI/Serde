use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::errors::{InventoryError, InventoryResult};

/// Supported currencies with custom serialization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
    CHF,
    CNY,
}

impl Currency {
    /// Get the number of decimal places for this currency
    pub fn decimal_places(&self) -> u32 {
        match self {
            Currency::JPY => 0, // Japanese Yen has no decimal places
            _ => 2,
        }
    }

    /// Get the currency symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
            Currency::JPY => "¥",
            Currency::CAD => "C$",
            Currency::AUD => "A$",
            Currency::CHF => "CHF",
            Currency::CNY => "¥",
        }
    }
}

/// Money type with currency and amount
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    /// Create a new Money instance
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Format as currency string
    pub fn format(&self) -> String {
        let decimal_places = self.currency.decimal_places() as usize;
        format!("{}{:.prec$}", self.currency.symbol(), self.amount, prec = decimal_places)
    }

    /// Convert to different currency (simplified exchange rate)
    pub fn convert_to(&self, target_currency: Currency, exchange_rate: Decimal) -> Money {
        Money::new(self.amount * exchange_rate, target_currency)
    }
}

/// Product classification for ABC analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductClass {
    A, // High value, low volume
    B, // Medium value, medium volume  
    C, // Low value, high volume
}

/// Product status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductStatus {
    Active,
    Discontinued,
    Seasonal,
    PreOrder,
    OutOfStock,
}

/// Comprehensive product model with custom serde implementations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub subcategory: Option<String>,
    pub unit_cost: Money,
    pub retail_price: Money,
    pub weight_grams: u32,
    pub dimensions_cm: (f64, f64, f64), // length, width, height
    pub barcode: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub minimum_stock: u32,
    pub maximum_stock: u32,
    pub reorder_point: u32,
    pub lead_time_days: u16,
    pub status: ProductStatus,
    pub classification: Option<ProductClass>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_metadata", deserialize_with = "deserialize_metadata")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Product {
    /// Calculate volume in cubic centimeters
    pub fn volume_cm3(&self) -> f64 {
        self.dimensions_cm.0 * self.dimensions_cm.1 * self.dimensions_cm.2
    }

    /// Calculate profit margin
    pub fn profit_margin(&self) -> InventoryResult<Decimal> {
        if self.unit_cost.currency != self.retail_price.currency {
            return Err(InventoryError::currency("Currency mismatch in profit calculation"));
        }
        
        if self.retail_price.amount == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        let profit = self.retail_price.amount - self.unit_cost.amount;
        Ok(profit / self.retail_price.amount * Decimal::from(100))
    }

    /// Check if product needs reordering based on current stock
    pub fn needs_reorder(&self, current_stock: u32) -> bool {
        current_stock <= self.reorder_point
    }

    /// Calculate recommended order quantity using Economic Order Quantity (EOQ) formula
    pub fn calculate_eoq(&self, annual_demand: u32, ordering_cost: Decimal) -> InventoryResult<u32> {
        if annual_demand == 0 {
            return Ok(0);
        }

        // EOQ = sqrt((2 * D * S) / H)
        // Where D = annual demand, S = ordering cost, H = holding cost per unit per year
        let holding_cost = self.unit_cost.amount * Decimal::from_f64_retain(0.25).unwrap(); // 25% holding cost
        
        let eoq_squared = (Decimal::from(2) * Decimal::from(annual_demand) * ordering_cost) / holding_cost;
        
        // Convert to string then parse to f64 for sqrt calculation
        let eoq_f64: f64 = eoq_squared.to_string().parse().map_err(|_| {
            InventoryError::calculation("Failed to convert decimal to f64 in EOQ calculation")
        })?;
        
        let eoq_result = eoq_f64.sqrt();
        let eoq = Decimal::from_f64_retain(eoq_result).ok_or_else(|| {
            InventoryError::calculation("Failed to calculate square root in EOQ formula")
        })?;

        // Convert decimal to u32 via string
        let eoq_u32: u32 = eoq.round().to_string().parse().unwrap_or(1);
        Ok(eoq_u32.max(1))
    }

    /// Validate product data integrity
    pub fn validate(&self) -> InventoryResult<()> {
        if self.sku.trim().is_empty() {
            return Err(InventoryError::validation("SKU cannot be empty"));
        }

        if self.name.trim().is_empty() {
            return Err(InventoryError::validation("Product name cannot be empty"));
        }

        if self.unit_cost.amount < Decimal::ZERO {
            return Err(InventoryError::validation("Unit cost cannot be negative"));
        }

        if self.retail_price.amount < Decimal::ZERO {
            return Err(InventoryError::validation("Retail price cannot be negative"));
        }

        if self.minimum_stock > self.maximum_stock {
            return Err(InventoryError::validation("Minimum stock cannot exceed maximum stock"));
        }

        if self.reorder_point < self.minimum_stock {
            return Err(InventoryError::validation("Reorder point should not be below minimum stock"));
        }

        Ok(())
    }
}

/// Supplier information with contact details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Supplier {
    pub id: Uuid,
    pub name: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub address: Address,
    pub payment_terms: String,
    pub lead_time_days: u16,
    pub quality_rating: f32, // 0.0 to 10.0
    pub reliability_score: f32, // 0.0 to 1.0
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Supplier {
    /// Check if supplier meets quality standards
    pub fn meets_quality_standards(&self, min_quality: f32, min_reliability: f32) -> bool {
        self.active && self.quality_rating >= min_quality && self.reliability_score >= min_reliability
    }
}

/// Address structure for suppliers and locations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub street_1: String,
    pub street_2: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country: String,
}

/// Storage location within a facility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub facility_id: String,
    pub zone: String,
    pub aisle: String,
    pub shelf: String,
    pub bin: String,
    pub capacity_units: u32,
    pub current_units: u32,
    pub location_type: LocationType,
    pub temperature_controlled: bool,
    pub hazmat_approved: bool,
}

impl Location {
    /// Calculate utilization percentage
    pub fn utilization_percentage(&self) -> f32 {
        if self.capacity_units == 0 {
            return 0.0;
        }
        (self.current_units as f32 / self.capacity_units as f32) * 100.0
    }

    /// Check if location has available capacity
    pub fn has_capacity(&self, units_needed: u32) -> bool {
        self.current_units + units_needed <= self.capacity_units
    }
}

/// Location type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    Receiving,
    Storage,
    Picking,
    Packing,
    Shipping,
    Quarantine,
    Returns,
}

/// Inventory transaction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Receipt,
    Shipment,
    Adjustment,
    Transfer,
    CycleCount,
    Damaged,
    Expired,
    Return,
}

/// Inventory transaction record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub transaction_type: TransactionType,
    pub quantity: i32, // Can be negative for outbound transactions
    pub unit_cost: Option<Money>,
    pub reference_number: Option<String>,
    pub reason_code: Option<String>,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub batch_number: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
}

impl Transaction {
    /// Check if transaction increases inventory
    pub fn is_inbound(&self) -> bool {
        matches!(self.transaction_type, TransactionType::Receipt | TransactionType::Return) 
        || (matches!(self.transaction_type, TransactionType::Adjustment | TransactionType::Transfer) && self.quantity > 0)
    }

    /// Check if transaction decreases inventory
    pub fn is_outbound(&self) -> bool {
        !self.is_inbound() && self.quantity != 0
    }
}

/// Current inventory snapshot for a product at a location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventorySnapshot {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub quantity_on_hand: u32,
    pub quantity_available: u32, // On hand minus reserved
    pub quantity_reserved: u32,
    pub quantity_on_order: u32,
    pub average_cost: Money,
    pub last_counted: Option<DateTime<Utc>>,
    pub last_movement: Option<DateTime<Utc>>,
    pub batch_numbers: Vec<String>,
}

impl InventorySnapshot {
    /// Calculate inventory value
    pub fn total_value(&self) -> Money {
        Money::new(
            self.average_cost.amount * Decimal::from(self.quantity_on_hand),
            self.average_cost.currency.clone()
        )
    }

    /// Check if inventory is available for allocation
    pub fn can_allocate(&self, quantity: u32) -> bool {
        self.quantity_available >= quantity
    }

    /// Reserve inventory
    pub fn reserve(&mut self, quantity: u32) -> InventoryResult<()> {
        if !self.can_allocate(quantity) {
            return Err(InventoryError::validation("Insufficient available inventory for reservation"));
        }

        self.quantity_available -= quantity;
        self.quantity_reserved += quantity;
        Ok(())
    }
}

// Custom serialization functions for metadata HashMap
fn serialize_metadata<S>(metadata: &HashMap<String, serde_json::Value>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::Error;
    serde_json::to_value(metadata)
        .map_err(S::Error::custom)?
        .serialize(serializer)
}

fn deserialize_metadata<'de, D>(deserializer: D) -> Result<HashMap<String, serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    serde_json::from_value(value).map_err(D::Error::custom)
}