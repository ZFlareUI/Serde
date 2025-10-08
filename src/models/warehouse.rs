//! Warehouse and location management models
//!
//! This module provides comprehensive warehouse management including
//! multi-location inventory, capacity planning, and operational status.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Physical address information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Address {
    /// Street address line 1
    pub street_1: String,
    /// Street address line 2 (optional)
    pub street_2: Option<String>,
    /// City
    pub city: String,
    /// State/Province
    pub state: String,
    /// Postal/ZIP code
    pub postal_code: String,
    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,
    /// Latitude for geolocation
    pub latitude: Option<Decimal>,
    /// Longitude for geolocation
    pub longitude: Option<Decimal>,
}

impl Address {
    /// Create a new address
    pub fn new(
        street_1: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
        postal_code: impl Into<String>,
        country: impl Into<String>,
    ) -> Self {
        Self {
            street_1: street_1.into(),
            street_2: None,
            city: city.into(),
            state: state.into(),
            postal_code: postal_code.into(),
            country: country.into(),
            latitude: None,
            longitude: None,
        }
    }

    /// Get formatted address string
    pub fn formatted(&self) -> String {
        let mut parts = vec![self.street_1.clone()];
        
        if let Some(ref street_2) = self.street_2 {
            if !street_2.trim().is_empty() {
                parts.push(street_2.clone());
            }
        }
        
        parts.push(format!("{}, {} {}", self.city, self.state, self.postal_code));
        parts.push(self.country.clone());
        
        parts.join("\n")
    }
}

/// Warehouse operational status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WarehouseStatus {
    Active,
    Inactive,
    Maintenance,
    Closed,
}

impl Default for WarehouseStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Warehouse type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WarehouseType {
    /// Main distribution center
    DistributionCenter,
    /// Regional fulfillment center
    FulfillmentCenter,
    /// Local warehouse
    Local,
    /// Cross-dock facility
    CrossDock,
    /// Returns processing center
    Returns,
    /// Manufacturing facility
    Manufacturing,
}

/// Warehouse capacity and utilization metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct WarehouseCapacity {
    /// Total storage capacity in cubic meters
    pub total_volume_m3: Decimal,
    /// Used storage volume in cubic meters
    pub used_volume_m3: Decimal,
    /// Maximum weight capacity in kilograms
    pub max_weight_kg: Decimal,
    /// Current weight in kilograms
    pub current_weight_kg: Decimal,
    /// Number of storage locations/bins
    pub total_locations: u32,
    /// Number of occupied locations
    pub occupied_locations: u32,
}

impl WarehouseCapacity {
    /// Create new warehouse capacity
    pub fn new(
        total_volume_m3: Decimal,
        max_weight_kg: Decimal,
        total_locations: u32,
    ) -> Self {
        Self {
            total_volume_m3,
            used_volume_m3: Decimal::ZERO,
            max_weight_kg,
            current_weight_kg: Decimal::ZERO,
            total_locations,
            occupied_locations: 0,
        }
    }

    /// Calculate volume utilization percentage
    pub fn volume_utilization_percent(&self) -> Decimal {
        if self.total_volume_m3 == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.used_volume_m3 / self.total_volume_m3) * Decimal::from(100)
    }

    /// Calculate weight utilization percentage
    pub fn weight_utilization_percent(&self) -> Decimal {
        if self.max_weight_kg == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.current_weight_kg / self.max_weight_kg) * Decimal::from(100)
    }

    /// Calculate location utilization percentage
    pub fn location_utilization_percent(&self) -> Decimal {
        if self.total_locations == 0 {
            return Decimal::ZERO;
        }
        (Decimal::from(self.occupied_locations) / Decimal::from(self.total_locations)) * Decimal::from(100)
    }

    /// Available volume
    pub fn available_volume_m3(&self) -> Decimal {
        self.total_volume_m3 - self.used_volume_m3
    }

    /// Available weight capacity
    pub fn available_weight_kg(&self) -> Decimal {
        self.max_weight_kg - self.current_weight_kg
    }

    /// Available locations
    pub fn available_locations(&self) -> u32 {
        self.total_locations - self.occupied_locations
    }
}

/// Operating hours for warehouse
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct OperatingHours {
    /// Operating hours by day of week (0=Sunday, 6=Saturday)
    pub hours: HashMap<u8, (String, String)>, // (open_time, close_time)
    /// Timezone (e.g., "UTC", "America/New_York")
    pub timezone: String,
    /// 24/7 operation flag
    pub is_24_7: bool,
}

impl Default for OperatingHours {
    fn default() -> Self {
        let mut hours = HashMap::new();
        // Default business hours Monday-Friday
        for day in 1..=5 {
            hours.insert(day, ("09:00".to_string(), "17:00".to_string()));
        }
        
        Self {
            hours,
            timezone: "UTC".to_string(),
            is_24_7: false,
        }
    }
}

/// Storage location within a warehouse
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct StorageLocation {
    /// Unique location identifier
    pub id: Uuid,
    /// Human-readable location code (e.g., "A1-B2-C3")
    pub code: String,
    /// Warehouse ID this location belongs to
    pub warehouse_id: Uuid,
    /// Zone identifier (e.g., "PICK", "BULK", "HAZMAT")
    pub zone: String,
    /// Aisle identifier
    pub aisle: String,
    /// Shelf/Bay identifier
    pub shelf: String,
    /// Bin/Position identifier
    pub bin: String,
    /// Location capacity in cubic meters
    pub capacity_m3: Decimal,
    /// Current utilization in cubic meters
    pub used_m3: Decimal,
    /// Temperature controlled flag
    pub temperature_controlled: bool,
    /// Target temperature range (if controlled)
    pub temperature_range: Option<(Decimal, Decimal)>,
    /// Special handling requirements
    pub special_requirements: Vec<String>,
    /// Location status
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl StorageLocation {
    /// Create a new storage location
    pub fn new(
        warehouse_id: Uuid,
        code: impl Into<String>,
        zone: impl Into<String>,
        aisle: impl Into<String>,
        shelf: impl Into<String>,
        bin: impl Into<String>,
        capacity_m3: Decimal,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            code: code.into(),
            warehouse_id,
            zone: zone.into(),
            aisle: aisle.into(),
            shelf: shelf.into(),
            bin: bin.into(),
            capacity_m3,
            used_m3: Decimal::ZERO,
            temperature_controlled: false,
            temperature_range: None,
            special_requirements: Vec::new(),
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate utilization percentage
    pub fn utilization_percent(&self) -> Decimal {
        if self.capacity_m3 == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.used_m3 / self.capacity_m3) * Decimal::from(100)
    }

    /// Available capacity
    pub fn available_capacity_m3(&self) -> Decimal {
        self.capacity_m3 - self.used_m3
    }

    /// Check if location can accommodate volume
    pub fn can_accommodate(&self, volume_m3: Decimal) -> bool {
        self.is_active && self.available_capacity_m3() >= volume_m3
    }
}

/// Main warehouse entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Warehouse {
    /// Unique warehouse identifier
    pub id: Uuid,
    
    /// Warehouse code/name
    pub code: String,
    
    /// Display name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Warehouse type
    pub warehouse_type: WarehouseType,
    
    /// Current status
    pub status: WarehouseStatus,
    
    /// Physical address
    pub address: Address,
    
    /// Capacity and utilization metrics
    pub capacity: WarehouseCapacity,
    
    /// Operating hours
    pub operating_hours: OperatingHours,
    
    /// Manager/contact information
    pub manager_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    
    /// Warehouse tags for categorization
    pub tags: Vec<String>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Warehouse {
    /// Create a new warehouse
    pub fn new(
        code: impl Into<String>,
        name: impl Into<String>,
        address: Address,
        capacity: WarehouseCapacity,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            code: code.into(),
            name: name.into(),
            description: None,
            warehouse_type: WarehouseType::Local,
            status: WarehouseStatus::default(),
            address,
            capacity,
            operating_hours: OperatingHours::default(),
            manager_name: None,
            contact_email: None,
            contact_phone: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Update the warehouse's updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Check if warehouse is operational
    pub fn is_operational(&self) -> bool {
        matches!(self.status, WarehouseStatus::Active)
    }

    /// Add a tag if it doesn't exist
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.touch();
        }
    }

    /// Check if warehouse is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.capacity.volume_utilization_percent() >= Decimal::from(95) ||
        self.capacity.weight_utilization_percent() >= Decimal::from(95) ||
        self.capacity.location_utilization_percent() >= Decimal::from(95)
    }
}

/// Builder for warehouse creation
#[derive(Debug, Default)]
pub struct WarehouseBuilder {
    code: Option<String>,
    name: Option<String>,
    description: Option<String>,
    warehouse_type: Option<WarehouseType>,
    status: Option<WarehouseStatus>,
    address: Option<Address>,
    capacity: Option<WarehouseCapacity>,
    operating_hours: Option<OperatingHours>,
    manager_name: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    tags: Vec<String>,
}

impl WarehouseBuilder {
    /// Create new warehouse builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set warehouse code
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set warehouse name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set warehouse type
    pub fn warehouse_type(mut self, warehouse_type: WarehouseType) -> Self {
        self.warehouse_type = Some(warehouse_type);
        self
    }

    /// Set status
    pub fn status(mut self, status: WarehouseStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set address
    pub fn address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }

    /// Set capacity
    pub fn capacity(mut self, capacity: WarehouseCapacity) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Set operating hours
    pub fn operating_hours(mut self, operating_hours: OperatingHours) -> Self {
        self.operating_hours = Some(operating_hours);
        self
    }

    /// Set manager name
    pub fn manager_name(mut self, manager_name: impl Into<String>) -> Self {
        self.manager_name = Some(manager_name.into());
        self
    }

    /// Set contact email
    pub fn contact_email(mut self, email: impl Into<String>) -> Self {
        self.contact_email = Some(email.into());
        self
    }

    /// Set contact phone
    pub fn contact_phone(mut self, phone: impl Into<String>) -> Self {
        self.contact_phone = Some(phone.into());
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Build the warehouse
    pub fn build(self) -> Result<Warehouse, &'static str> {
        let code = self.code.ok_or("Code is required")?;
        let name = self.name.ok_or("Name is required")?;
        let address = self.address.ok_or("Address is required")?;
        let capacity = self.capacity.ok_or("Capacity is required")?;

        let now = Utc::now();
        
        Ok(Warehouse {
            id: Uuid::new_v4(),
            code,
            name,
            description: self.description,
            warehouse_type: self.warehouse_type.unwrap_or(WarehouseType::Local),
            status: self.status.unwrap_or_default(),
            address,
            capacity,
            operating_hours: self.operating_hours.unwrap_or_default(),
            manager_name: self.manager_name,
            contact_email: self.contact_email,
            contact_phone: self.contact_phone,
            tags: self.tags,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_address_formatting() {
        let mut address = Address::new(
            "123 Main St",
            "Anytown",
            "CA",
            "90210",
            "US"
        );
        address.street_2 = Some("Suite 100".to_string());
        
        let formatted = address.formatted();
        assert!(formatted.contains("123 Main St"));
        assert!(formatted.contains("Suite 100"));
        assert!(formatted.contains("Anytown, CA 90210"));
    }

    #[test]
    fn test_warehouse_capacity() {
        let capacity = WarehouseCapacity::new(dec!(1000.0), dec!(50000.0), 500);
        
        assert_eq!(capacity.volume_utilization_percent(), Decimal::ZERO);
        assert_eq!(capacity.available_volume_m3(), dec!(1000.0));
        assert_eq!(capacity.available_locations(), 500);
    }

    #[test]
    fn test_storage_location() {
        let warehouse_id = Uuid::new_v4();
        let location = StorageLocation::new(
            warehouse_id,
            "A1-B2-C3",
            "PICK",
            "A1",
            "B2",
            "C3",
            dec!(2.5),
        );

        assert_eq!(location.warehouse_id, warehouse_id);
        assert_eq!(location.code, "A1-B2-C3");
        assert!(location.can_accommodate(dec!(2.0)));
        assert!(!location.can_accommodate(dec!(3.0)));
    }

    #[test]
    fn test_warehouse_builder() {
        let address = Address::new("123 Warehouse Rd", "Industrial City", "TX", "12345", "US");
        let capacity = WarehouseCapacity::new(dec!(5000.0), dec!(100000.0), 1000);
        
        let warehouse = WarehouseBuilder::new()
            .code("WH001")
            .name("Main Warehouse")
            .address(address)
            .capacity(capacity)
            .warehouse_type(WarehouseType::DistributionCenter)
            .tag("main")
            .build()
            .unwrap();

        assert_eq!(warehouse.code, "WH001");
        assert_eq!(warehouse.name, "Main Warehouse");
        assert!(warehouse.is_operational());
        assert!(!warehouse.is_at_capacity());
    }
}