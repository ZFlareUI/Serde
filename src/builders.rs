use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::{Product, Supplier, Location, Money, Currency, ProductStatus, LocationType, Address};
use crate::errors::{InventoryError, InventoryResult};

/// Builder pattern for creating products with validation
#[derive(Debug, Clone)]
pub struct ProductBuilder {
    id: Uuid,
    sku: Option<String>,
    name: Option<String>,
    description: Option<String>,
    category: Option<String>,
    subcategory: Option<String>,
    unit_cost: Option<Money>,
    retail_price: Option<Money>,
    weight_grams: Option<u32>,
    dimensions_cm: Option<(f64, f64, f64)>,
    barcode: Option<String>,
    supplier_id: Option<Uuid>,
    minimum_stock: Option<u32>,
    maximum_stock: Option<u32>,
    reorder_point: Option<u32>,
    lead_time_days: Option<u16>,
    status: Option<ProductStatus>,
    tags: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl ProductBuilder {
    /// Create a new product builder with required fields
    pub fn new<S1, S2>(sku: S1, name: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            id: Uuid::new_v4(),
            sku: Some(sku.into()),
            name: Some(name.into()),
            description: None,
            category: None,
            subcategory: None,
            unit_cost: None,
            retail_price: None,
            weight_grams: None,
            dimensions_cm: None,
            barcode: None,
            supplier_id: None,
            minimum_stock: None,
            maximum_stock: None,
            reorder_point: None,
            lead_time_days: None,
            status: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set product ID (useful for importing existing data)
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    /// Set product description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set product category
    pub fn category<S: Into<String>>(mut self, category: S) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set product subcategory
    pub fn subcategory<S: Into<String>>(mut self, subcategory: S) -> Self {
        self.subcategory = Some(subcategory.into());
        self
    }

    /// Set unit cost with automatic retail price calculation if not set
    pub fn unit_cost(mut self, amount: Decimal, currency: Currency) -> Self {
        self.unit_cost = Some(Money::new(amount, currency.clone()));
        
        // Auto-set retail price with 50% markup if not already set
        if self.retail_price.is_none() {
            let markup_multiplier = Decimal::from_f64_retain(1.5).unwrap(); // 50% markup
            self.retail_price = Some(Money::new(amount * markup_multiplier, currency));
        }
        self
    }

    /// Set retail price
    pub fn retail_price(mut self, amount: Decimal, currency: Currency) -> Self {
        self.retail_price = Some(Money::new(amount, currency));
        self
    }

    /// Set weight in grams
    pub fn weight_grams(mut self, weight: u32) -> Self {
        self.weight_grams = Some(weight);
        self
    }

    /// Set dimensions in centimeters (length, width, height)
    pub fn dimensions_cm(mut self, length: f64, width: f64, height: f64) -> Self {
        self.dimensions_cm = Some((length, width, height));
        self
    }

    /// Set barcode
    pub fn barcode<S: Into<String>>(mut self, barcode: S) -> Self {
        self.barcode = Some(barcode.into());
        self
    }

    /// Set supplier ID
    pub fn supplier_id(mut self, supplier_id: Uuid) -> Self {
        self.supplier_id = Some(supplier_id);
        self
    }

    /// Set stock levels with automatic validation
    pub fn stock_levels(mut self, minimum: u32, maximum: u32, reorder_point: u32) -> InventoryResult<Self> {
        if minimum > maximum {
            return Err(InventoryError::builder("Minimum stock cannot exceed maximum stock"));
        }
        
        if reorder_point < minimum {
            return Err(InventoryError::builder("Reorder point should not be below minimum stock"));
        }

        if reorder_point > maximum {
            return Err(InventoryError::builder("Reorder point should not exceed maximum stock"));
        }

        self.minimum_stock = Some(minimum);
        self.maximum_stock = Some(maximum);
        self.reorder_point = Some(reorder_point);
        Ok(self)
    }

    /// Set lead time in days
    pub fn lead_time_days(mut self, days: u16) -> Self {
        self.lead_time_days = Some(days);
        self
    }

    /// Set product status
    pub fn status(mut self, status: ProductStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Add a tag
    pub fn add_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn add_tags<I, S>(mut self, tags: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(|tag| tag.into()));
        self
    }

    /// Add metadata
    pub fn add_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the product with validation
    pub fn build(self) -> InventoryResult<Product> {
        let sku = self.sku.ok_or_else(|| InventoryError::builder("SKU is required"))?;
        let name = self.name.ok_or_else(|| InventoryError::builder("Product name is required"))?;
        let category = self.category.ok_or_else(|| InventoryError::builder("Category is required"))?;
        let unit_cost = self.unit_cost.ok_or_else(|| InventoryError::builder("Unit cost is required"))?;
        let retail_price = self.retail_price.ok_or_else(|| InventoryError::builder("Retail price is required"))?;

        // Ensure currencies match
        if unit_cost.currency != retail_price.currency {
            return Err(InventoryError::builder("Unit cost and retail price must use the same currency"));
        }

        // Set reasonable defaults
        let minimum_stock = self.minimum_stock.unwrap_or(10);
        let maximum_stock = self.maximum_stock.unwrap_or(100);
        let reorder_point = self.reorder_point.unwrap_or(minimum_stock * 2);

        // Validate stock levels
        if minimum_stock > maximum_stock {
            return Err(InventoryError::builder("Minimum stock cannot exceed maximum stock"));
        }

        let now = Utc::now();
        
        let product = Product {
            id: self.id,
            sku,
            name,
            description: self.description,
            category,
            subcategory: self.subcategory,
            unit_cost,
            retail_price,
            weight_grams: self.weight_grams.unwrap_or(0),
            dimensions_cm: self.dimensions_cm.unwrap_or((0.0, 0.0, 0.0)),
            barcode: self.barcode,
            supplier_id: self.supplier_id,
            minimum_stock,
            maximum_stock,
            reorder_point,
            lead_time_days: self.lead_time_days.unwrap_or(7),
            status: self.status.unwrap_or(ProductStatus::Active),
            classification: None, // Set by ABC analysis
            tags: self.tags,
            created_at: now,
            updated_at: now,
            metadata: self.metadata,
        };

        product.validate()?;
        Ok(product)
    }
}

/// Builder pattern for creating suppliers
#[derive(Debug, Clone)]
pub struct SupplierBuilder {
    id: Uuid,
    name: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    address: Option<Address>,
    payment_terms: Option<String>,
    lead_time_days: Option<u16>,
    quality_rating: Option<f32>,
    reliability_score: Option<f32>,
    active: bool,
}

impl SupplierBuilder {
    /// Create a new supplier builder
    pub fn new<S1, S2>(name: S1, contact_email: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            id: Uuid::new_v4(),
            name: Some(name.into()),
            contact_email: Some(contact_email.into()),
            contact_phone: None,
            address: None,
            payment_terms: None,
            lead_time_days: None,
            quality_rating: None,
            reliability_score: None,
            active: true,
        }
    }

    /// Set supplier ID
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    /// Set contact phone
    pub fn contact_phone<S: Into<String>>(mut self, phone: S) -> Self {
        self.contact_phone = Some(phone.into());
        self
    }

    /// Set address using AddressBuilder
    pub fn address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }

    /// Set payment terms
    pub fn payment_terms<S: Into<String>>(mut self, terms: S) -> Self {
        self.payment_terms = Some(terms.into());
        self
    }

    /// Set lead time in days
    pub fn lead_time_days(mut self, days: u16) -> Self {
        self.lead_time_days = Some(days);
        self
    }

    /// Set quality rating (0.0 to 10.0)
    pub fn quality_rating(mut self, rating: f32) -> InventoryResult<Self> {
        if !(0.0..=10.0).contains(&rating) {
            return Err(InventoryError::builder("Quality rating must be between 0.0 and 10.0"));
        }
        self.quality_rating = Some(rating);
        Ok(self)
    }

    /// Set reliability score (0.0 to 1.0)
    pub fn reliability_score(mut self, score: f32) -> InventoryResult<Self> {
        if !(0.0..=1.0).contains(&score) {
            return Err(InventoryError::builder("Reliability score must be between 0.0 and 1.0"));
        }
        self.reliability_score = Some(score);
        Ok(self)
    }

    /// Set active status
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Build the supplier
    pub fn build(self) -> InventoryResult<Supplier> {
        let name = self.name.ok_or_else(|| InventoryError::builder("Supplier name is required"))?;
        let contact_email = self.contact_email.ok_or_else(|| InventoryError::builder("Contact email is required"))?;
        let address = self.address.ok_or_else(|| InventoryError::builder("Address is required"))?;

        // Validate email format (basic check)
        if !contact_email.contains('@') {
            return Err(InventoryError::builder("Invalid email format"));
        }

        let now = Utc::now();

        Ok(Supplier {
            id: self.id,
            name,
            contact_email,
            contact_phone: self.contact_phone,
            address,
            payment_terms: self.payment_terms.unwrap_or_else(|| "Net 30".to_string()),
            lead_time_days: self.lead_time_days.unwrap_or(14),
            quality_rating: self.quality_rating.unwrap_or(5.0),
            reliability_score: self.reliability_score.unwrap_or(0.8),
            active: self.active,
            created_at: now,
            updated_at: now,
        })
    }
}

/// Builder pattern for creating addresses
#[derive(Debug, Clone)]
pub struct AddressBuilder {
    street_1: Option<String>,
    street_2: Option<String>,
    city: Option<String>,
    state_province: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

impl AddressBuilder {
    /// Create a new address builder
    pub fn new() -> Self {
        Self {
            street_1: None,
            street_2: None,
            city: None,
            state_province: None,
            postal_code: None,
            country: None,
        }
    }

    /// Set street address line 1
    pub fn street_1<S: Into<String>>(mut self, street: S) -> Self {
        self.street_1 = Some(street.into());
        self
    }

    /// Set street address line 2
    pub fn street_2<S: Into<String>>(mut self, street: S) -> Self {
        self.street_2 = Some(street.into());
        self
    }

    /// Set city
    pub fn city<S: Into<String>>(mut self, city: S) -> Self {
        self.city = Some(city.into());
        self
    }

    /// Set state or province
    pub fn state_province<S: Into<String>>(mut self, state_province: S) -> Self {
        self.state_province = Some(state_province.into());
        self
    }

    /// Set postal code
    pub fn postal_code<S: Into<String>>(mut self, postal_code: S) -> Self {
        self.postal_code = Some(postal_code.into());
        self
    }

    /// Set country
    pub fn country<S: Into<String>>(mut self, country: S) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Build the address
    pub fn build(self) -> InventoryResult<Address> {
        let street_1 = self.street_1.ok_or_else(|| InventoryError::builder("Street address is required"))?;
        let city = self.city.ok_or_else(|| InventoryError::builder("City is required"))?;
        let state_province = self.state_province.ok_or_else(|| InventoryError::builder("State/Province is required"))?;
        let postal_code = self.postal_code.ok_or_else(|| InventoryError::builder("Postal code is required"))?;
        let country = self.country.ok_or_else(|| InventoryError::builder("Country is required"))?;

        Ok(Address {
            street_1,
            street_2: self.street_2,
            city,
            state_province,
            postal_code,
            country,
        })
    }
}

impl Default for AddressBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating storage locations
#[derive(Debug, Clone)]
pub struct LocationBuilder {
    id: Uuid,
    facility_id: Option<String>,
    zone: Option<String>,
    aisle: Option<String>,
    shelf: Option<String>,
    bin: Option<String>,
    capacity_units: Option<u32>,
    location_type: Option<LocationType>,
    temperature_controlled: bool,
    hazmat_approved: bool,
}

impl LocationBuilder {
    /// Create a new location builder
    pub fn new<S: Into<String>>(facility_id: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            facility_id: Some(facility_id.into()),
            zone: None,
            aisle: None,
            shelf: None,
            bin: None,
            capacity_units: None,
            location_type: None,
            temperature_controlled: false,
            hazmat_approved: false,
        }
    }

    /// Set location ID
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    /// Set zone
    pub fn zone<S: Into<String>>(mut self, zone: S) -> Self {
        self.zone = Some(zone.into());
        self
    }

    /// Set aisle
    pub fn aisle<S: Into<String>>(mut self, aisle: S) -> Self {
        self.aisle = Some(aisle.into());
        self
    }

    /// Set shelf
    pub fn shelf<S: Into<String>>(mut self, shelf: S) -> Self {
        self.shelf = Some(shelf.into());
        self
    }

    /// Set bin
    pub fn bin<S: Into<String>>(mut self, bin: S) -> Self {
        self.bin = Some(bin.into());
        self
    }

    /// Set capacity in units
    pub fn capacity_units(mut self, capacity: u32) -> Self {
        self.capacity_units = Some(capacity);
        self
    }

    /// Set location type
    pub fn location_type(mut self, location_type: LocationType) -> Self {
        self.location_type = Some(location_type);
        self
    }

    /// Set temperature controlled flag
    pub fn temperature_controlled(mut self, controlled: bool) -> Self {
        self.temperature_controlled = controlled;
        self
    }

    /// Set hazmat approved flag
    pub fn hazmat_approved(mut self, approved: bool) -> Self {
        self.hazmat_approved = approved;
        self
    }

    /// Build the location
    pub fn build(self) -> InventoryResult<Location> {
        let facility_id = self.facility_id.ok_or_else(|| InventoryError::builder("Facility ID is required"))?;
        let zone = self.zone.ok_or_else(|| InventoryError::builder("Zone is required"))?;
        let aisle = self.aisle.ok_or_else(|| InventoryError::builder("Aisle is required"))?;
        let shelf = self.shelf.ok_or_else(|| InventoryError::builder("Shelf is required"))?;
        let bin = self.bin.ok_or_else(|| InventoryError::builder("Bin is required"))?;

        Ok(Location {
            id: self.id,
            facility_id,
            zone,
            aisle,
            shelf,
            bin,
            capacity_units: self.capacity_units.unwrap_or(1000),
            current_units: 0, // Start empty
            location_type: self.location_type.unwrap_or(LocationType::Storage),
            temperature_controlled: self.temperature_controlled,
            hazmat_approved: self.hazmat_approved,
        })
    }
}

/// Fluent interface for building complex inventory setups
#[derive(Debug)]
pub struct InventorySetupBuilder {
    pub products: Vec<Product>,
    pub suppliers: Vec<Supplier>,
    pub locations: Vec<Location>,
}

impl InventorySetupBuilder {
    /// Create a new inventory setup builder
    pub fn new() -> Self {
        Self {
            products: Vec::new(),
            suppliers: Vec::new(),
            locations: Vec::new(),
        }
    }

    /// Add a product
    pub fn add_product(mut self, product: Product) -> Self {
        self.products.push(product);
        self
    }

    /// Add multiple products
    pub fn add_products<I>(mut self, products: I) -> Self
    where
        I: IntoIterator<Item = Product>,
    {
        self.products.extend(products);
        self
    }

    /// Add a supplier
    pub fn add_supplier(mut self, supplier: Supplier) -> Self {
        self.suppliers.push(supplier);
        self
    }

    /// Add multiple suppliers
    pub fn add_suppliers<I>(mut self, suppliers: I) -> Self
    where
        I: IntoIterator<Item = Supplier>,
    {
        self.suppliers.extend(suppliers);
        self
    }

    /// Add a location
    pub fn add_location(mut self, location: Location) -> Self {
        self.locations.push(location);
        self
    }

    /// Add multiple locations
    pub fn add_locations<I>(mut self, locations: I) -> Self
    where
        I: IntoIterator<Item = Location>,
    {
        self.locations.extend(locations);
        self
    }

    /// Validate the entire setup for consistency
    pub fn validate(&self) -> InventoryResult<()> {
        // Check for duplicate SKUs
        let mut skus = std::collections::HashSet::new();
        for product in &self.products {
            if !skus.insert(&product.sku) {
                return Err(InventoryError::validation(format!("Duplicate SKU found: {}", product.sku)));
            }
        }

        // Check supplier references
        let supplier_ids: std::collections::HashSet<Uuid> = self.suppliers.iter().map(|s| s.id).collect();
        for product in &self.products {
            if let Some(supplier_id) = product.supplier_id {
                if !supplier_ids.contains(&supplier_id) {
                    return Err(InventoryError::validation(format!("Product {} references non-existent supplier", product.sku)));
                }
            }
        }

        // Validate individual products
        for product in &self.products {
            product.validate()?;
        }

        Ok(())
    }

    /// Build and return the complete setup
    pub fn build(self) -> InventoryResult<InventorySetup> {
        self.validate()?;
        Ok(InventorySetup {
            products: self.products,
            suppliers: self.suppliers,
            locations: self.locations,
        })
    }
}

impl Default for InventorySetupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete inventory setup result
#[derive(Debug)]
pub struct InventorySetup {
    pub products: Vec<Product>,
    pub suppliers: Vec<Supplier>,
    pub locations: Vec<Location>,
}

impl InventorySetup {
    /// Get products by supplier
    pub fn products_by_supplier(&self, supplier_id: Uuid) -> Vec<&Product> {
        self.products.iter()
            .filter(|p| p.supplier_id == Some(supplier_id))
            .collect()
    }

    /// Get products by category
    pub fn products_by_category(&self, category: &str) -> Vec<&Product> {
        self.products.iter()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get all categories
    pub fn categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.products.iter()
            .map(|p| p.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// Calculate total inventory value
    pub fn total_inventory_value(&self) -> HashMap<Currency, Decimal> {
        let mut totals = HashMap::new();
        
        for product in &self.products {
            let current_value = product.unit_cost.amount * Decimal::from(product.minimum_stock);
            *totals.entry(product.unit_cost.currency.clone()).or_insert(Decimal::ZERO) += current_value;
        }

        totals
    }
}