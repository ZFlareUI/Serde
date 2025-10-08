//! Domain models for inventory management
//!
//! This module contains all the core business entities used throughout
//! the inventory management system, including products, warehouses,
//! suppliers, transactions, and orders.

pub mod product;
pub mod warehouse;
pub mod transaction;
pub mod supplier;
pub mod order;

// Re-export main types for convenience
pub use product::{Product, ProductBuilder, ProductCategory, ProductStatus, Pricing, Dimensions, InventoryLevels};
pub use warehouse::{Warehouse, WarehouseBuilder, WarehouseType, WarehouseStatus, Address, WarehouseCapacity, StorageLocation};
pub use transaction::{StockTransaction, StockTransactionBuilder, TransactionType, TransactionStatus, BatchInfo, TransactionFinancials};

/// Common types used across all models
pub mod common {
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "schema")]
    use schemars::JsonSchema;

    /// Standard currency representation
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    pub struct Currency {
        /// ISO 4217 currency code
        pub code: String,
        /// Display name
        pub name: String,
        /// Symbol (e.g., $, €, £)
        pub symbol: String,
        /// Number of decimal places
        pub decimal_places: u8,
    }

    impl Currency {
        /// Create new currency
        pub fn new(code: impl Into<String>, name: impl Into<String>, symbol: impl Into<String>, decimal_places: u8) -> Self {
            Self {
                code: code.into(),
                name: name.into(),
                symbol: symbol.into(),
                decimal_places,
            }
        }

        /// US Dollar
        pub fn usd() -> Self {
            Self::new("USD", "US Dollar", "$", 2)
        }

        /// Euro
        pub fn eur() -> Self {
            Self::new("EUR", "Euro", "€", 2)
        }

        /// British Pound
        pub fn gbp() -> Self {
            Self::new("GBP", "British Pound", "£", 2)
        }
    }

    /// Money value with currency
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    pub struct Money {
        /// Amount
        pub amount: Decimal,
        /// Currency
        pub currency: Currency,
    }

    impl Money {
        /// Create new money value
        pub fn new(amount: Decimal, currency: Currency) -> Self {
            Self { amount, currency }
        }

        /// Format as string with currency symbol
        pub fn format(&self) -> String {
            format!("{}{:.prec$}", self.currency.symbol, self.amount, prec = self.currency.decimal_places as usize)
        }

        /// Convert to different currency (requires exchange rate)
        pub fn convert_to(&self, target_currency: Currency, exchange_rate: Decimal) -> Money {
            Money::new(self.amount * exchange_rate, target_currency)
        }
    }

    /// Contact information
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    pub struct ContactInfo {
        /// Contact name
        pub name: Option<String>,
        /// Email address
        pub email: Option<String>,
        /// Phone number
        pub phone: Option<String>,
        /// Fax number
        pub fax: Option<String>,
        /// Website URL
        pub website: Option<String>,
    }

    impl ContactInfo {
        /// Create new contact info
        pub fn new() -> Self {
            Self {
                name: None,
                email: None,
                phone: None,
                fax: None,
                website: None,
            }
        }

        /// Check if any contact method is available
        pub fn has_contact_method(&self) -> bool {
            self.email.is_some() || self.phone.is_some()
        }
    }

    impl Default for ContactInfo {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Audit trail for entity changes
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    pub struct AuditInfo {
        /// Who created the entity
        pub created_by: Option<String>,
        /// When the entity was created
        pub created_at: DateTime<Utc>,
        /// Who last updated the entity
        pub updated_by: Option<String>,
        /// When the entity was last updated
        pub updated_at: DateTime<Utc>,
        /// Version number for optimistic locking
        pub version: i64,
    }

    impl AuditInfo {
        /// Create new audit info
        pub fn new(user_id: Option<String>) -> Self {
            let now = Utc::now();
            Self {
                created_by: user_id.clone(),
                created_at: now,
                updated_by: user_id,
                updated_at: now,
                version: 1,
            }
        }

        /// Update audit info for a change
        pub fn update(&mut self, user_id: Option<String>) {
            self.updated_by = user_id;
            self.updated_at = Utc::now();
            self.version += 1;
        }
    }

    /// Status enumeration for entities
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    #[serde(rename_all = "snake_case")]
    pub enum EntityStatus {
        Active,
        Inactive,
        Pending,
        Suspended,
        Deleted,
    }

    impl Default for EntityStatus {
        fn default() -> Self {
            Self::Active
        }
    }

    /// Geographic coordinate
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    pub struct GeoCoordinate {
        /// Latitude
        pub latitude: Decimal,
        /// Longitude
        pub longitude: Decimal,
        /// Altitude in meters (optional)
        pub altitude: Option<Decimal>,
    }

    impl GeoCoordinate {
        /// Create new coordinate
        pub fn new(latitude: Decimal, longitude: Decimal) -> Self {
            Self {
                latitude,
                longitude,
                altitude: None,
            }
        }

        /// Calculate distance to another coordinate (Haversine formula)
        pub fn distance_to(&self, other: &GeoCoordinate) -> Decimal {
            use std::f64::consts::PI;
            
            let lat1 = self.latitude.to_f64().unwrap() * PI / 180.0;
            let lat2 = other.latitude.to_f64().unwrap() * PI / 180.0;
            let dlat = (other.latitude - self.latitude).to_f64().unwrap() * PI / 180.0;
            let dlon = (other.longitude - self.longitude).to_f64().unwrap() * PI / 180.0;

            let a = (dlat / 2.0).sin().powi(2) +
                    lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
            let c = 2.0_f64 * a.sqrt().atan2((1.0_f64 - a).sqrt());

            // Earth's radius in kilometers
            Decimal::from_f64_retain(6371.0 * c).unwrap_or_default()
        }
    }

    /// Unit of measure
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    pub struct UnitOfMeasure {
        /// Code (e.g., EA, KG, M)
        pub code: String,
        /// Display name
        pub name: String,
        /// Abbreviation for display
        pub abbreviation: String,
        /// Type category
        pub category: UnitCategory,
    }

    /// Unit of measure categories
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema", derive(JsonSchema))]
    #[serde(rename_all = "snake_case")]
    pub enum UnitCategory {
        Count,
        Weight,
        Volume,
        Length,
        Area,
        Time,
    }

    impl UnitOfMeasure {
        /// Each (piece)
        pub fn each() -> Self {
            Self {
                code: "EA".to_string(),
                name: "Each".to_string(),
                abbreviation: "ea".to_string(),
                category: UnitCategory::Count,
            }
        }

        /// Kilogram
        pub fn kilogram() -> Self {
            Self {
                code: "KG".to_string(),
                name: "Kilogram".to_string(),
                abbreviation: "kg".to_string(),
                category: UnitCategory::Weight,
            }
        }

        /// Liter
        pub fn liter() -> Self {
            Self {
                code: "L".to_string(),
                name: "Liter".to_string(),
                abbreviation: "l".to_string(),
                category: UnitCategory::Volume,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rust_decimal_macros::dec;

        #[test]
        fn test_currency() {
            let usd = Currency::usd();
            assert_eq!(usd.code, "USD");
            assert_eq!(usd.symbol, "$");
        }

        #[test]
        fn test_money_formatting() {
            let money = Money::new(dec!(123.45), Currency::usd());
            assert_eq!(money.format(), "$123.45");
        }

        #[test]
        fn test_geo_coordinate_distance() {
            // New York to London (approximate)
            let ny = GeoCoordinate::new(dec!(40.7128), dec!(-74.0060));
            let london = GeoCoordinate::new(dec!(51.5074), dec!(-0.1278));
            
            let distance = ny.distance_to(&london);
            // Should be approximately 5585 km
            assert!(distance > dec!(5500) && distance < dec!(5600));
        }
    }
}