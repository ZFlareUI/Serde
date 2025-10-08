//! Stock transaction models for inventory tracking
//!
//! This module provides comprehensive transaction tracking for all
//! inventory movements including purchases, sales, transfers, and adjustments.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Transaction type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Inbound stock from supplier
    Receipt,
    /// Outbound stock to customer
    Sale,
    /// Stock transfer between warehouses
    Transfer,
    /// Stock adjustment (positive or negative)
    Adjustment,
    /// Stock return from customer
    Return,
    /// Stock reserved for future shipment
    Reservation,
    /// Cancel previous reservation
    ReservationCancel,
    /// Manufacturing consumption
    Production,
    /// Quality control hold
    QualityHold,
    /// Release from quality hold
    QualityRelease,
    /// Stock write-off/loss
    WriteOff,
    /// Cycle count adjustment
    CycleCount,
}

/// Transaction status tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// Transaction is planned but not executed
    Planned,
    /// Transaction is in progress
    InProgress,
    /// Transaction completed successfully
    Completed,
    /// Transaction was cancelled
    Cancelled,
    /// Transaction failed and needs attention
    Failed,
    /// Transaction is pending approval
    PendingApproval,
}

impl Default for TransactionStatus {
    fn default() -> Self {
        Self::Planned
    }
}

/// Cost method for inventory valuation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CostMethod {
    /// First In, First Out
    Fifo,
    /// Last In, First Out
    Lifo,
    /// Weighted Average Cost
    WeightedAverage,
    /// Standard Cost
    Standard,
    /// Specific Identification
    Specific,
}

impl Default for CostMethod {
    fn default() -> Self {
        Self::WeightedAverage
    }
}

/// Batch/lot tracking information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct BatchInfo {
    /// Batch or lot number
    pub batch_number: String,
    /// Expiration date (if applicable)
    pub expiry_date: Option<DateTime<Utc>>,
    /// Manufacturing date
    pub manufactured_date: Option<DateTime<Utc>>,
    /// Supplier batch reference
    pub supplier_batch: Option<String>,
    /// Quality status
    pub quality_status: String,
}

impl BatchInfo {
    /// Create new batch info
    pub fn new(batch_number: impl Into<String>) -> Self {
        Self {
            batch_number: batch_number.into(),
            expiry_date: None,
            manufactured_date: None,
            supplier_batch: None,
            quality_status: "approved".to_string(),
        }
    }

    /// Check if batch is expired
    pub fn is_expired(&self) -> bool {
        self.expiry_date
            .map(|expiry| expiry < Utc::now())
            .unwrap_or(false)
    }

    /// Days until expiry (if applicable)
    pub fn days_to_expiry(&self) -> Option<i64> {
        self.expiry_date.map(|expiry| {
            let now = Utc::now();
            (expiry - now).num_days()
        })
    }
}

/// Financial information for transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct TransactionFinancials {
    /// Unit cost at transaction time
    pub unit_cost: Decimal,
    /// Total cost for the transaction
    pub total_cost: Decimal,
    /// Currency code (ISO 4217)
    pub currency: String,
    /// Cost method used
    pub cost_method: CostMethod,
    /// Exchange rate (if different from base currency)
    pub exchange_rate: Option<Decimal>,
    /// Tax amount
    pub tax_amount: Option<Decimal>,
    /// Discount amount
    pub discount_amount: Option<Decimal>,
}

impl TransactionFinancials {
    /// Create new transaction financials
    pub fn new(unit_cost: Decimal, quantity: Decimal, currency: impl Into<String>) -> Self {
        Self {
            unit_cost,
            total_cost: unit_cost * quantity,
            currency: currency.into(),
            cost_method: CostMethod::default(),
            exchange_rate: None,
            tax_amount: None,
            discount_amount: None,
        }
    }

    /// Calculate net total (including tax and discount)
    pub fn net_total(&self) -> Decimal {
        let mut total = self.total_cost;
        
        if let Some(tax) = self.tax_amount {
            total += tax;
        }
        
        if let Some(discount) = self.discount_amount {
            total -= discount;
        }
        
        total
    }
}

/// Core stock transaction entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct StockTransaction {
    /// Unique transaction identifier
    pub id: Uuid,
    
    /// Transaction type
    pub transaction_type: TransactionType,
    
    /// Current status
    pub status: TransactionStatus,
    
    /// Product being transacted
    pub product_id: Uuid,
    
    /// Source warehouse (for outbound/transfers)
    pub source_warehouse_id: Option<Uuid>,
    
    /// Destination warehouse (for inbound/transfers)
    pub destination_warehouse_id: Option<Uuid>,
    
    /// Source location within warehouse
    pub source_location_id: Option<Uuid>,
    
    /// Destination location within warehouse
    pub destination_location_id: Option<Uuid>,
    
    /// Quantity being transacted (positive for inbound, negative for outbound)
    pub quantity: Decimal,
    
    /// Unit of measure
    pub unit_of_measure: String,
    
    /// Financial information
    pub financials: Option<TransactionFinancials>,
    
    /// Batch/lot information
    pub batch_info: Option<BatchInfo>,
    
    /// Reference to external document (PO, SO, etc.)
    pub external_reference: Option<String>,
    
    /// Internal reference number
    pub reference_number: Option<String>,
    
    /// Notes or comments
    pub notes: Option<String>,
    
    /// User who initiated the transaction
    pub user_id: Option<String>,
    
    /// Reason code for the transaction
    pub reason_code: Option<String>,
    
    /// Related transaction ID (for reversals, etc.)
    pub related_transaction_id: Option<Uuid>,
    
    /// Planned execution time
    pub planned_at: Option<DateTime<Utc>>,
    
    /// Actual execution time
    pub executed_at: Option<DateTime<Utc>>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl StockTransaction {
    /// Create a new stock transaction
    pub fn new(
        transaction_type: TransactionType,
        product_id: Uuid,
        quantity: Decimal,
        unit_of_measure: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            transaction_type,
            status: TransactionStatus::default(),
            product_id,
            source_warehouse_id: None,
            destination_warehouse_id: None,
            source_location_id: None,
            destination_location_id: None,
            quantity,
            unit_of_measure: unit_of_measure.into(),
            financials: None,
            batch_info: None,
            external_reference: None,
            reference_number: None,
            notes: None,
            user_id: None,
            reason_code: None,
            related_transaction_id: None,
            planned_at: None,
            executed_at: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Update the transaction's updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Check if transaction increases stock
    pub fn is_inbound(&self) -> bool {
        matches!(
            self.transaction_type,
            TransactionType::Receipt
                | TransactionType::Return
                | TransactionType::ReservationCancel
                | TransactionType::QualityRelease
        ) || (matches!(self.transaction_type, TransactionType::Adjustment) && self.quantity > Decimal::ZERO)
    }

    /// Check if transaction decreases stock
    pub fn is_outbound(&self) -> bool {
        matches!(
            self.transaction_type,
            TransactionType::Sale
                | TransactionType::Reservation
                | TransactionType::Production
                | TransactionType::QualityHold
                | TransactionType::WriteOff
        ) || (matches!(self.transaction_type, TransactionType::Adjustment) && self.quantity < Decimal::ZERO)
    }

    /// Check if transaction is a transfer between locations
    pub fn is_transfer(&self) -> bool {
        matches!(self.transaction_type, TransactionType::Transfer)
    }

    /// Mark transaction as completed
    pub fn complete(&mut self) -> Result<(), &'static str> {
        if !matches!(self.status, TransactionStatus::Planned | TransactionStatus::InProgress) {
            return Err("Transaction cannot be completed from current status");
        }
        
        self.status = TransactionStatus::Completed;
        self.executed_at = Some(Utc::now());
        self.touch();
        
        Ok(())
    }

    /// Cancel the transaction
    pub fn cancel(&mut self, reason: Option<String>) -> Result<(), &'static str> {
        if matches!(self.status, TransactionStatus::Completed) {
            return Err("Cannot cancel completed transaction");
        }
        
        self.status = TransactionStatus::Cancelled;
        if let Some(reason) = reason {
            self.reason_code = Some(reason);
        }
        self.touch();
        
        Ok(())
    }

    /// Create a reversal transaction
    pub fn create_reversal(&self, reason: Option<String>) -> StockTransaction {
        let mut reversal = StockTransaction::new(
            self.transaction_type.clone(),
            self.product_id,
            -self.quantity, // Reverse the quantity
            self.unit_of_measure.clone(),
        );
        
        reversal.source_warehouse_id = self.destination_warehouse_id;
        reversal.destination_warehouse_id = self.source_warehouse_id;
        reversal.source_location_id = self.destination_location_id;
        reversal.destination_location_id = self.source_location_id;
        reversal.related_transaction_id = Some(self.id);
        reversal.reason_code = reason;
        reversal.financials = self.financials.clone();
        reversal.batch_info = self.batch_info.clone();
        
        reversal
    }
}

/// Builder for stock transactions
#[derive(Debug)]
pub struct StockTransactionBuilder {
    transaction_type: Option<TransactionType>,
    product_id: Option<Uuid>,
    quantity: Option<Decimal>,
    unit_of_measure: Option<String>,
    source_warehouse_id: Option<Uuid>,
    destination_warehouse_id: Option<Uuid>,
    source_location_id: Option<Uuid>,
    destination_location_id: Option<Uuid>,
    financials: Option<TransactionFinancials>,
    batch_info: Option<BatchInfo>,
    external_reference: Option<String>,
    reference_number: Option<String>,
    notes: Option<String>,
    user_id: Option<String>,
    reason_code: Option<String>,
    planned_at: Option<DateTime<Utc>>,
}

impl StockTransactionBuilder {
    /// Create new transaction builder
    pub fn new() -> Self {
        Self {
            transaction_type: None,
            product_id: None,
            quantity: None,
            unit_of_measure: None,
            source_warehouse_id: None,
            destination_warehouse_id: None,
            source_location_id: None,
            destination_location_id: None,
            financials: None,
            batch_info: None,
            external_reference: None,
            reference_number: None,
            notes: None,
            user_id: None,
            reason_code: None,
            planned_at: None,
        }
    }

    /// Set transaction type
    pub fn transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.transaction_type = Some(transaction_type);
        self
    }

    /// Set product ID
    pub fn product_id(mut self, product_id: Uuid) -> Self {
        self.product_id = Some(product_id);
        self
    }

    /// Set quantity
    pub fn quantity(mut self, quantity: Decimal) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Set unit of measure
    pub fn unit_of_measure(mut self, unit: impl Into<String>) -> Self {
        self.unit_of_measure = Some(unit.into());
        self
    }

    /// Set source warehouse
    pub fn source_warehouse(mut self, warehouse_id: Uuid) -> Self {
        self.source_warehouse_id = Some(warehouse_id);
        self
    }

    /// Set destination warehouse
    pub fn destination_warehouse(mut self, warehouse_id: Uuid) -> Self {
        self.destination_warehouse_id = Some(warehouse_id);
        self
    }

    /// Set source location
    pub fn source_location(mut self, location_id: Uuid) -> Self {
        self.source_location_id = Some(location_id);
        self
    }

    /// Set destination location
    pub fn destination_location(mut self, location_id: Uuid) -> Self {
        self.destination_location_id = Some(location_id);
        self
    }

    /// Set financials
    pub fn financials(mut self, financials: TransactionFinancials) -> Self {
        self.financials = Some(financials);
        self
    }

    /// Set batch info
    pub fn batch_info(mut self, batch_info: BatchInfo) -> Self {
        self.batch_info = Some(batch_info);
        self
    }

    /// Set external reference
    pub fn external_reference(mut self, reference: impl Into<String>) -> Self {
        self.external_reference = Some(reference.into());
        self
    }

    /// Set reference number
    pub fn reference_number(mut self, reference: impl Into<String>) -> Self {
        self.reference_number = Some(reference.into());
        self
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set user ID
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set reason code
    pub fn reason_code(mut self, reason: impl Into<String>) -> Self {
        self.reason_code = Some(reason.into());
        self
    }

    /// Set planned execution time
    pub fn planned_at(mut self, planned_at: DateTime<Utc>) -> Self {
        self.planned_at = Some(planned_at);
        self
    }

    /// Build the transaction
    pub fn build(self) -> Result<StockTransaction, &'static str> {
        let transaction_type = self.transaction_type.ok_or("Transaction type is required")?;
        let product_id = self.product_id.ok_or("Product ID is required")?;
        let quantity = self.quantity.ok_or("Quantity is required")?;
        let unit_of_measure = self.unit_of_measure.ok_or("Unit of measure is required")?;

        let now = Utc::now();
        
        Ok(StockTransaction {
            id: Uuid::new_v4(),
            transaction_type,
            status: TransactionStatus::default(),
            product_id,
            source_warehouse_id: self.source_warehouse_id,
            destination_warehouse_id: self.destination_warehouse_id,
            source_location_id: self.source_location_id,
            destination_location_id: self.destination_location_id,
            quantity,
            unit_of_measure,
            financials: self.financials,
            batch_info: self.batch_info,
            external_reference: self.external_reference,
            reference_number: self.reference_number,
            notes: self.notes,
            user_id: self.user_id,
            reason_code: self.reason_code,
            related_transaction_id: None,
            planned_at: self.planned_at,
            executed_at: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        })
    }
}

impl Default for StockTransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_transaction_creation() {
        let product_id = Uuid::new_v4();
        
        let transaction = StockTransactionBuilder::new()
            .transaction_type(TransactionType::Sale)
            .product_id(product_id)
            .quantity(dec!(10.0))
            .unit_of_measure("EA")
            .build()
            .unwrap();

        assert_eq!(transaction.product_id, product_id);
        assert_eq!(transaction.quantity, dec!(10.0));
        assert!(transaction.is_outbound());
        assert!(!transaction.is_inbound());
    }

    #[test]
    fn test_transaction_completion() {
        let mut transaction = StockTransaction::new(
            TransactionType::Receipt,
            Uuid::new_v4(),
            dec!(5.0),
            "EA",
        );

        assert!(transaction.complete().is_ok());
        assert_eq!(transaction.status, TransactionStatus::Completed);
        assert!(transaction.executed_at.is_some());
    }

    #[test]
    fn test_batch_info() {
        let mut batch = BatchInfo::new("BATCH-001");
        batch.expiry_date = Some(Utc::now() - chrono::Duration::days(1));
        
        assert!(batch.is_expired());
        assert!(batch.days_to_expiry().unwrap() < 0);
    }

    #[test]
    fn test_transaction_financials() {
        let financials = TransactionFinancials::new(dec!(10.0), dec!(5.0), "USD");
        
        assert_eq!(financials.total_cost, dec!(50.0));
        assert_eq!(financials.net_total(), dec!(50.0));
    }

    #[test]
    fn test_transaction_reversal() {
        let product_id = Uuid::new_v4();
        let warehouse_id = Uuid::new_v4();
        
        let original = StockTransactionBuilder::new()
            .transaction_type(TransactionType::Sale)
            .product_id(product_id)
            .quantity(dec!(10.0))
            .unit_of_measure("EA")
            .destination_warehouse(warehouse_id)
            .build()
            .unwrap();

        let reversal = original.create_reversal(Some("Customer return".to_string()));
        
        assert_eq!(reversal.quantity, dec!(-10.0));
        assert_eq!(reversal.related_transaction_id, Some(original.id));
        assert_eq!(reversal.source_warehouse_id, Some(warehouse_id));
    }
}