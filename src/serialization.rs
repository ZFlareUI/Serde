use std::collections::HashMap;
use serde_json;
use csv::{ReaderBuilder, WriterBuilder};
use uuid::Uuid;
use crate::models::{Product, Supplier, InventorySnapshot, Transaction, Currency};
use crate::algorithms::{InventorySystem, ReorderRecommendation};
use crate::errors::{InventoryError, InventoryResult};

/// Trait for serializing data to different formats
pub trait FormatSerializer<T> {
    /// Serialize to JSON string
    fn to_json(&self) -> InventoryResult<String>;
    
    /// Serialize to TOML string
    fn to_toml(&self) -> InventoryResult<String>;
    
    /// Serialize to CSV string
    fn to_csv(&self) -> InventoryResult<String>;
    
    /// Deserialize from JSON string
    fn from_json(json: &str) -> InventoryResult<T>;
    
    /// Deserialize from TOML string
    fn from_toml(toml: &str) -> InventoryResult<T>;
    
    /// Deserialize from CSV string
    fn from_csv(csv: &str) -> InventoryResult<T>;
}

/// Implementation for single Product
impl FormatSerializer<Product> for Product {
    fn to_json(&self) -> InventoryResult<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    fn to_toml(&self) -> InventoryResult<String> {
        toml::to_string(self).map_err(Into::into)
    }

    fn to_csv(&self) -> InventoryResult<String> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        
        // Custom CSV format for products
        wtr.write_record(&[
            "id", "sku", "name", "category", "unit_cost_amount", "unit_cost_currency",
            "retail_price_amount", "retail_price_currency", "weight_grams", "minimum_stock",
            "maximum_stock", "reorder_point", "status"
        ])?;

        wtr.write_record(&[
            &self.id.to_string(),
            &self.sku,
            &self.name,
            &self.category,
            &self.unit_cost.amount.to_string(),
            &format!("{:?}", self.unit_cost.currency),
            &self.retail_price.amount.to_string(),
            &format!("{:?}", self.retail_price.currency),
            &self.weight_grams.to_string(),
            &self.minimum_stock.to_string(),
            &self.maximum_stock.to_string(),
            &self.reorder_point.to_string(),
            &format!("{:?}", self.status),
        ])?;

        let data = String::from_utf8(wtr.into_inner().map_err(|e| InventoryError::serialization(format!("CSV writer error: {:?}", e)))?)?;
        Ok(data)
    }

    fn from_json(json: &str) -> InventoryResult<Product> {
        serde_json::from_str(json).map_err(Into::into)
    }

    fn from_toml(toml_str: &str) -> InventoryResult<Product> {
        toml::from_str(toml_str).map_err(Into::into)
    }

    fn from_csv(_csv: &str) -> InventoryResult<Product> {
        Err(InventoryError::serialization("CSV deserialization not implemented for single Product - use ProductList instead"))
    }
}

/// Wrapper for collections to implement serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductList {
    pub products: Vec<Product>,
}

impl ProductList {
    pub fn new(products: Vec<Product>) -> Self {
        Self { products }
    }
}

impl FormatSerializer<ProductList> for ProductList {
    fn to_json(&self) -> InventoryResult<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    fn to_toml(&self) -> InventoryResult<String> {
        toml::to_string(self).map_err(Into::into)
    }

    fn to_csv(&self) -> InventoryResult<String> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        
        // Write headers
        wtr.write_record(&[
            "id", "sku", "name", "category", "subcategory", "unit_cost_amount", "unit_cost_currency",
            "retail_price_amount", "retail_price_currency", "weight_grams", "dimensions_length",
            "dimensions_width", "dimensions_height", "minimum_stock", "maximum_stock", "reorder_point",
            "lead_time_days", "status", "tags"
        ])?;

        // Write product data
        for product in &self.products {
            wtr.write_record(&[
                &product.id.to_string(),
                &product.sku,
                &product.name,
                &product.category,
                product.subcategory.as_deref().unwrap_or(""),
                &product.unit_cost.amount.to_string(),
                &format!("{:?}", product.unit_cost.currency),
                &product.retail_price.amount.to_string(),
                &format!("{:?}", product.retail_price.currency),
                &product.weight_grams.to_string(),
                &product.dimensions_cm.0.to_string(),
                &product.dimensions_cm.1.to_string(),
                &product.dimensions_cm.2.to_string(),
                &product.minimum_stock.to_string(),
                &product.maximum_stock.to_string(),
                &product.reorder_point.to_string(),
                &product.lead_time_days.to_string(),
                &format!("{:?}", product.status),
                &product.tags.join(";"),
            ])?;
        }

        let data = String::from_utf8(wtr.into_inner().map_err(|e| InventoryError::serialization(format!("CSV writer error: {:?}", e)))?)?;
        Ok(data)
    }

    fn from_json(json: &str) -> InventoryResult<ProductList> {
        serde_json::from_str(json).map_err(Into::into)
    }

    fn from_toml(toml_str: &str) -> InventoryResult<ProductList> {
        toml::from_str(toml_str).map_err(Into::into)
    }

    fn from_csv(csv_data: &str) -> InventoryResult<ProductList> {
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(csv_data.as_bytes());
        let mut products = Vec::new();

        for result in rdr.records() {
            let record = result?;
            
            if record.len() < 18 {
                return Err(InventoryError::serialization("Invalid CSV format - insufficient columns"));
            }

            let id = Uuid::parse_str(&record[0])
                .map_err(|_| InventoryError::serialization("Invalid UUID format in CSV"))?;
            
            let unit_cost_amount = record[5].parse()
                .map_err(|_| InventoryError::serialization("Invalid unit cost amount in CSV"))?;
            
            let unit_cost_currency = match &record[6] {
                "USD" => Currency::USD,
                "EUR" => Currency::EUR,
                "GBP" => Currency::GBP,
                "JPY" => Currency::JPY,
                "CAD" => Currency::CAD,
                "AUD" => Currency::AUD,
                "CHF" => Currency::CHF,
                "CNY" => Currency::CNY,
                _ => return Err(InventoryError::serialization("Invalid currency in CSV")),
            };

            let retail_price_amount = record[7].parse()
                .map_err(|_| InventoryError::serialization("Invalid retail price amount in CSV"))?;
            
            let retail_price_currency = match &record[8] {
                "USD" => Currency::USD,
                "EUR" => Currency::EUR,
                "GBP" => Currency::GBP,
                "JPY" => Currency::JPY,
                "CAD" => Currency::CAD,
                "AUD" => Currency::AUD,
                "CHF" => Currency::CHF,
                "CNY" => Currency::CNY,
                _ => return Err(InventoryError::serialization("Invalid currency in CSV")),
            };

            // Parse other fields with error handling
            let weight_grams = record[9].parse().unwrap_or(0);
            let dimensions_length = record[10].parse().unwrap_or(0.0);
            let dimensions_width = record[11].parse().unwrap_or(0.0);
            let dimensions_height = record[12].parse().unwrap_or(0.0);
            let minimum_stock = record[13].parse().unwrap_or(0);
            let maximum_stock = record[14].parse().unwrap_or(100);
            let reorder_point = record[15].parse().unwrap_or(minimum_stock);
            let lead_time_days = record[16].parse().unwrap_or(7);

            let status = match &record[17] {
                "Active" => crate::models::ProductStatus::Active,
                "Discontinued" => crate::models::ProductStatus::Discontinued,
                "Seasonal" => crate::models::ProductStatus::Seasonal,
                "PreOrder" => crate::models::ProductStatus::PreOrder,
                "OutOfStock" => crate::models::ProductStatus::OutOfStock,
                _ => crate::models::ProductStatus::Active,
            };

            let tags = if record.len() > 18 && !record[18].is_empty() {
                record[18].split(';').map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            };

            let product = Product {
                id,
                sku: record[1].to_string(),
                name: record[2].to_string(),
                description: None,
                category: record[3].to_string(),
                subcategory: if record[4].is_empty() { None } else { Some(record[4].to_string()) },
                unit_cost: crate::models::Money::new(unit_cost_amount, unit_cost_currency),
                retail_price: crate::models::Money::new(retail_price_amount, retail_price_currency),
                weight_grams,
                dimensions_cm: (dimensions_length, dimensions_width, dimensions_height),
                barcode: None,
                supplier_id: None,
                minimum_stock,
                maximum_stock,
                reorder_point,
                lead_time_days,
                status,
                classification: None,
                tags,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            };

            products.push(product);
        }

        Ok(ProductList { products })
    }
}

/// Wrapper for supplier collections
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SupplierList {
    pub suppliers: Vec<Supplier>,
}

impl SupplierList {
    pub fn new(suppliers: Vec<Supplier>) -> Self {
        Self { suppliers }
    }
}

impl FormatSerializer<SupplierList> for SupplierList {
    fn to_json(&self) -> InventoryResult<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    fn to_toml(&self) -> InventoryResult<String> {
        toml::to_string(self).map_err(Into::into)
    }

    fn to_csv(&self) -> InventoryResult<String> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        
        wtr.write_record(&[
            "id", "name", "contact_email", "contact_phone", "street_1", "city", 
            "state_province", "postal_code", "country", "payment_terms", "lead_time_days",
            "quality_rating", "reliability_score", "active"
        ])?;

        for supplier in &self.suppliers {
            wtr.write_record(&[
                &supplier.id.to_string(),
                &supplier.name,
                &supplier.contact_email,
                supplier.contact_phone.as_deref().unwrap_or(""),
                &supplier.address.street_1,
                &supplier.address.city,
                &supplier.address.state_province,
                &supplier.address.postal_code,
                &supplier.address.country,
                &supplier.payment_terms,
                &supplier.lead_time_days.to_string(),
                &supplier.quality_rating.to_string(),
                &supplier.reliability_score.to_string(),
                &supplier.active.to_string(),
            ])?;
        }

        let data = String::from_utf8(wtr.into_inner().map_err(|e| InventoryError::serialization(format!("CSV writer error: {:?}", e)))?)?;
        Ok(data)
    }

    fn from_json(json: &str) -> InventoryResult<SupplierList> {
        serde_json::from_str(json).map_err(Into::into)
    }

    fn from_toml(toml_str: &str) -> InventoryResult<SupplierList> {
        toml::from_str(toml_str).map_err(Into::into)
    }

    fn from_csv(_csv: &str) -> InventoryResult<SupplierList> {
        Err(InventoryError::serialization("CSV deserialization for suppliers not yet implemented"))
    }
}

/// Comprehensive inventory system serialization
impl FormatSerializer<InventorySystem> for InventorySystem {
    fn to_json(&self) -> InventoryResult<String> {
        #[derive(serde::Serialize)]
        struct SerializableSystem<'a> {
            products: &'a HashMap<Uuid, Product>,
            inventory: &'a HashMap<(Uuid, Uuid), InventorySnapshot>,
            transactions: &'a Vec<Transaction>,
        }

        let serializable = SerializableSystem {
            products: &self.products,
            inventory: &self.inventory,
            transactions: &self.transactions,
        };

        serde_json::to_string_pretty(&serializable).map_err(Into::into)
    }

    fn to_toml(&self) -> InventoryResult<String> {
        // TOML doesn't handle complex keys well, so we'll serialize differently
        #[derive(serde::Serialize)]
        struct TomlSystem {
            products: Vec<Product>,
            inventory: Vec<TomlInventorySnapshot>,
            transactions: Vec<Transaction>,
        }

        #[derive(serde::Serialize)]
        struct TomlInventorySnapshot {
            product_id: Uuid,
            location_id: Uuid,
            quantity_on_hand: u32,
            quantity_available: u32,
            quantity_reserved: u32,
            quantity_on_order: u32,
            average_cost: crate::models::Money,
        }

        let toml_system = TomlSystem {
            products: self.products.values().cloned().collect(),
            inventory: self.inventory.iter().map(|((product_id, location_id), snapshot)| {
                TomlInventorySnapshot {
                    product_id: *product_id,
                    location_id: *location_id,
                    quantity_on_hand: snapshot.quantity_on_hand,
                    quantity_available: snapshot.quantity_available,
                    quantity_reserved: snapshot.quantity_reserved,
                    quantity_on_order: snapshot.quantity_on_order,
                    average_cost: snapshot.average_cost.clone(),
                }
            }).collect(),
            transactions: self.transactions.clone(),
        };

        toml::to_string(&toml_system).map_err(Into::into)
    }

    fn to_csv(&self) -> InventoryResult<String> {
        // For CSV, we'll export products only as the main entity
        let product_list = ProductList {
            products: self.products.values().cloned().collect(),
        };
        product_list.to_csv()
    }

    fn from_json(_json: &str) -> InventoryResult<InventorySystem> {
        Err(InventoryError::serialization("InventorySystem JSON deserialization not implemented - use individual components"))
    }

    fn from_toml(_toml: &str) -> InventoryResult<InventorySystem> {
        Err(InventoryError::serialization("InventorySystem TOML deserialization not implemented - use individual components"))
    }

    fn from_csv(_csv: &str) -> InventoryResult<InventorySystem> {
        Err(InventoryError::serialization("InventorySystem CSV deserialization not implemented - use individual components"))
    }
}

/// Wrapper for reorder recommendations with serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReorderRecommendationList {
    pub recommendations: Vec<ReorderRecommendation>,
}

impl ReorderRecommendationList {
    pub fn new(recommendations: Vec<ReorderRecommendation>) -> Self {
        Self { recommendations }
    }
}

impl FormatSerializer<ReorderRecommendationList> for ReorderRecommendationList {
    fn to_json(&self) -> InventoryResult<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    fn to_toml(&self) -> InventoryResult<String> {
        toml::to_string(self).map_err(Into::into)
    }

    fn to_csv(&self) -> InventoryResult<String> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        
        wtr.write_record(&[
            "product_id", "current_stock", "reorder_point", "recommended_quantity",
            "safety_stock", "predicted_demand", "annual_demand", "confidence_level",
            "trend_factor", "urgency"
        ])?;

        for rec in &self.recommendations {
            wtr.write_record(&[
                &rec.product_id.to_string(),
                &rec.current_stock.to_string(),
                &rec.reorder_point.to_string(),
                &rec.recommended_quantity.to_string(),
                &rec.safety_stock.to_string(),
                &rec.demand_forecast.predicted_demand.to_string(),
                &rec.demand_forecast.annual_demand.to_string(),
                &rec.demand_forecast.confidence_level.to_string(),
                &rec.demand_forecast.trend_factor.to_string(),
                &rec.urgency.to_string(),
            ])?;
        }

        let data = String::from_utf8(wtr.into_inner().map_err(|e| InventoryError::serialization(format!("CSV writer error: {:?}", e)))?)?;
        Ok(data)
    }

    fn from_json(json: &str) -> InventoryResult<ReorderRecommendationList> {
        serde_json::from_str(json).map_err(Into::into)
    }

    fn from_toml(toml_str: &str) -> InventoryResult<ReorderRecommendationList> {
        toml::from_str(toml_str).map_err(Into::into)
    }

    fn from_csv(_csv: &str) -> InventoryResult<ReorderRecommendationList> {
        Err(InventoryError::serialization("ReorderRecommendation CSV deserialization not implemented"))
    }
}

/// Enhanced serialization utilities
pub struct SerializationUtils;

impl SerializationUtils {
    /// Convert between different formats
    pub fn convert_format<T>(
        input: &str,
        from_format: SerializationFormat,
        to_format: SerializationFormat,
    ) -> InventoryResult<String>
    where
        T: FormatSerializer<T>,
    {
        let data = match from_format {
            SerializationFormat::Json => T::from_json(input)?,
            SerializationFormat::Toml => T::from_toml(input)?,
            SerializationFormat::Csv => T::from_csv(input)?,
        };

        match to_format {
            SerializationFormat::Json => data.to_json(),
            SerializationFormat::Toml => data.to_toml(),
            SerializationFormat::Csv => data.to_csv(),
        }
    }

    /// Validate JSON format
    pub fn validate_json(json: &str) -> InventoryResult<()> {
        serde_json::from_str::<serde_json::Value>(json)
            .map_err(|e| InventoryError::serialization(format!("Invalid JSON: {}", e)))?;
        Ok(())
    }

    /// Validate TOML format
    pub fn validate_toml(toml: &str) -> InventoryResult<()> {
        toml::from_str::<toml::Value>(toml)
            .map_err(|e| InventoryError::serialization(format!("Invalid TOML: {}", e)))?;
        Ok(())
    }

    /// Get format from file extension
    pub fn format_from_extension(filename: &str) -> Option<SerializationFormat> {
        match filename.split('.').last()?.to_lowercase().as_str() {
            "json" => Some(SerializationFormat::Json),
            "toml" => Some(SerializationFormat::Toml),
            "csv" => Some(SerializationFormat::Csv),
            _ => None,
        }
    }

    /// Auto-detect format from content
    pub fn auto_detect_format(content: &str) -> SerializationFormat {
        let trimmed = content.trim_start();
        
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            SerializationFormat::Json
        } else if trimmed.contains('=') && (trimmed.contains('[') || trimmed.lines().count() > 1) {
            SerializationFormat::Toml
        } else {
            SerializationFormat::Csv
        }
    }
}

/// Serialization format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    Toml,
    Csv,
}

impl std::fmt::Display for SerializationFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationFormat::Json => write!(f, "JSON"),
            SerializationFormat::Toml => write!(f, "TOML"),
            SerializationFormat::Csv => write!(f, "CSV"),
        }
    }
}

// Implement From<String> for InventoryError to handle UTF8 errors
impl From<std::string::FromUtf8Error> for InventoryError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::serialization(format!("UTF-8 conversion error: {}", err))
    }
}