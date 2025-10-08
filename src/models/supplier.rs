//! Supplier domain model
//!
//! This module defines the Supplier entity and related types for
//! managing supply chain relationships, including contact information,
//! payment terms, and performance tracking.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "schema")]
use schemars::JsonSchema;

use super::common::{ContactInfo, Money, Currency, AuditInfo, GeoCoordinate};
use super::warehouse::Address;

/// Supplier entity representing a company or individual that provides products
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Supplier {
    /// Unique identifier
    pub id: Uuid,
    /// Supplier code (business identifier)
    pub code: String,
    /// Company or supplier name
    pub name: String,
    /// Display name or trading name
    pub display_name: Option<String>,
    /// Supplier type
    pub supplier_type: SupplierType,
    /// Current status
    pub status: SupplierStatus,
    
    // Contact Information
    /// Primary contact information
    pub contact_info: ContactInfo,
    /// Billing address
    pub billing_address: Option<Address>,
    /// Shipping address
    pub shipping_address: Option<Address>,
    /// Geographic coordinates
    pub coordinates: Option<GeoCoordinate>,
    
    // Business Information
    /// Tax identification number
    pub tax_id: Option<String>,
    /// Business registration number
    pub registration_number: Option<String>,
    /// Primary currency for transactions
    pub currency: Currency,
    /// Payment terms
    pub payment_terms: PaymentTerms,
    
    // Performance Tracking
    /// Performance metrics
    pub performance: SupplierPerformance,
    /// Credit limit
    pub credit_limit: Option<Money>,
    /// Current balance owed to supplier
    pub current_balance: Money,
    
    // Metadata
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Types of suppliers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SupplierType {
    /// Primary manufacturer
    Manufacturer,
    /// Wholesale distributor
    Distributor,
    /// Drop-ship supplier
    Dropship,
    /// Service provider
    Service,
    /// Individual contractor
    Individual,
    /// Government entity
    Government,
}

/// Supplier status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SupplierStatus {
    /// Active and available for orders
    Active,
    /// Inactive but not deleted
    Inactive,
    /// Pending approval
    Pending,
    /// Suspended due to performance issues
    Suspended,
    /// Preferred supplier
    Preferred,
    /// Blocked from new orders
    Blocked,
}

impl Default for SupplierStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Payment terms for supplier transactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PaymentTerms {
    /// Net payment days (e.g., Net 30)
    pub net_days: u32,
    /// Early payment discount percentage
    pub discount_percentage: Option<Decimal>,
    /// Days to qualify for discount
    pub discount_days: Option<u32>,
    /// Late payment fee percentage
    pub late_fee_percentage: Option<Decimal>,
    /// Payment method preferences
    pub preferred_methods: Vec<PaymentMethod>,
}

/// Payment methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Check,
    BankTransfer,
    CreditCard,
    Ach,
    Wire,
    Cash,
}

impl PaymentTerms {
    /// Create standard Net 30 terms
    pub fn net_30() -> Self {
        Self {
            net_days: 30,
            discount_percentage: None,
            discount_days: None,
            late_fee_percentage: None,
            preferred_methods: vec![PaymentMethod::Check, PaymentMethod::BankTransfer],
        }
    }

    /// Create 2/10 Net 30 terms (2% discount if paid within 10 days)
    pub fn two_ten_net_30() -> Self {
        use rust_decimal_macros::dec;
        Self {
            net_days: 30,
            discount_percentage: Some(dec!(2.0)),
            discount_days: Some(10),
            late_fee_percentage: None,
            preferred_methods: vec![PaymentMethod::Check, PaymentMethod::BankTransfer],
        }
    }

    /// Calculate discount amount for early payment
    pub fn calculate_discount(&self, invoice_amount: &Money) -> Option<Money> {
        if let (Some(discount_pct), Some(_discount_days)) = (self.discount_percentage, self.discount_days) {
            let discount_amount = invoice_amount.amount * (discount_pct / rust_decimal_macros::dec!(100.0));
            Some(Money::new(discount_amount, invoice_amount.currency.clone()))
        } else {
            None
        }
    }
}

/// Supplier performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SupplierPerformance {
    /// Overall rating (1-5)
    pub overall_rating: Option<Decimal>,
    /// Quality rating (1-5)
    pub quality_rating: Option<Decimal>,
    /// Delivery rating (1-5)
    pub delivery_rating: Option<Decimal>,
    /// Service rating (1-5)
    pub service_rating: Option<Decimal>,
    
    /// On-time delivery percentage
    pub on_time_delivery_pct: Option<Decimal>,
    /// Quality acceptance percentage
    pub quality_acceptance_pct: Option<Decimal>,
    /// Average lead time in days
    pub avg_lead_time_days: Option<Decimal>,
    
    /// Total orders placed
    pub total_orders: u64,
    /// Total order value
    pub total_order_value: Money,
    /// Number of quality issues
    pub quality_issues: u32,
    /// Number of delivery delays
    pub delivery_delays: u32,
    
    /// Last review date
    pub last_review_date: Option<DateTime<Utc>>,
    /// Performance history by month
    pub monthly_performance: HashMap<String, MonthlyPerformance>,
}

/// Monthly performance snapshot
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct MonthlyPerformance {
    /// Month (YYYY-MM format)
    pub month: String,
    /// Orders placed that month
    pub orders_count: u32,
    /// Total value ordered
    pub order_value: Money,
    /// On-time deliveries
    pub on_time_deliveries: u32,
    /// Quality issues
    pub quality_issues: u32,
    /// Average rating for the month
    pub avg_rating: Option<Decimal>,
}

impl SupplierPerformance {
    /// Create new performance tracker
    pub fn new(currency: Currency) -> Self {
        Self {
            overall_rating: None,
            quality_rating: None,
            delivery_rating: None,
            service_rating: None,
            on_time_delivery_pct: None,
            quality_acceptance_pct: None,
            avg_lead_time_days: None,
            total_orders: 0,
            total_order_value: Money::new(rust_decimal_macros::dec!(0), currency),
            quality_issues: 0,
            delivery_delays: 0,
            last_review_date: None,
            monthly_performance: HashMap::new(),
        }
    }

    /// Update performance with new order data
    pub fn update_with_order(&mut self, order_value: Money, on_time: bool, quality_ok: bool) {
        self.total_orders += 1;
        self.total_order_value.amount += order_value.amount;
        
        if !on_time {
            self.delivery_delays += 1;
        }
        
        if !quality_ok {
            self.quality_issues += 1;
        }
        
        self.recalculate_ratings();
    }

    /// Recalculate performance ratings
    pub fn recalculate_ratings(&mut self) {
        use rust_decimal_macros::dec;
        
        if self.total_orders > 0 {
            // Calculate on-time delivery percentage
            let on_time_count = self.total_orders - self.delivery_delays as u64;
            self.on_time_delivery_pct = Some(
                (Decimal::from(on_time_count) / Decimal::from(self.total_orders)) * dec!(100)
            );
            
            // Calculate quality acceptance percentage
            let quality_ok_count = self.total_orders - self.quality_issues as u64;
            self.quality_acceptance_pct = Some(
                (Decimal::from(quality_ok_count) / Decimal::from(self.total_orders)) * dec!(100)
            );
            
            // Calculate delivery rating (based on on-time percentage)
            if let Some(on_time_pct) = self.on_time_delivery_pct {
                self.delivery_rating = Some(
                    if on_time_pct >= dec!(95) { dec!(5.0) }
                    else if on_time_pct >= dec!(85) { dec!(4.0) }
                    else if on_time_pct >= dec!(70) { dec!(3.0) }
                    else if on_time_pct >= dec!(50) { dec!(2.0) }
                    else { dec!(1.0) }
                );
            }
            
            // Calculate quality rating (based on acceptance percentage)
            if let Some(quality_pct) = self.quality_acceptance_pct {
                self.quality_rating = Some(
                    if quality_pct >= dec!(98) { dec!(5.0) }
                    else if quality_pct >= dec!(95) { dec!(4.0) }
                    else if quality_pct >= dec!(90) { dec!(3.0) }
                    else if quality_pct >= dec!(80) { dec!(2.0) }
                    else { dec!(1.0) }
                );
            }
            
            // Calculate overall rating (average of available ratings)
            let mut rating_sum = dec!(0);
            let mut rating_count = 0;
            
            if let Some(delivery) = self.delivery_rating {
                rating_sum += delivery;
                rating_count += 1;
            }
            
            if let Some(quality) = self.quality_rating {
                rating_sum += quality;
                rating_count += 1;
            }
            
            if let Some(service) = self.service_rating {
                rating_sum += service;
                rating_count += 1;
            }
            
            if rating_count > 0 {
                self.overall_rating = Some(rating_sum / Decimal::from(rating_count));
            }
        }
    }

    /// Check if supplier meets performance criteria
    pub fn meets_criteria(&self, min_rating: Decimal, min_on_time_pct: Decimal) -> bool {
        let rating_ok = self.overall_rating
            .map(|rating| rating >= min_rating)
            .unwrap_or(false);
        
        let on_time_ok = self.on_time_delivery_pct
            .map(|pct| pct >= min_on_time_pct)
            .unwrap_or(false);
        
        rating_ok && on_time_ok
    }
}

/// Builder for creating suppliers
pub struct SupplierBuilder {
    supplier: Supplier,
}

impl SupplierBuilder {
    /// Create new supplier builder
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self {
        let code = code.into();
        let name = name.into();
        
        Self {
            supplier: Supplier {
                id: Uuid::new_v4(),
                code: code.clone(),
                name: name.clone(),
                display_name: None,
                supplier_type: SupplierType::Distributor,
                status: SupplierStatus::default(),
                contact_info: ContactInfo::default(),
                billing_address: None,
                shipping_address: None,
                coordinates: None,
                tax_id: None,
                registration_number: None,
                currency: Currency::usd(),
                payment_terms: PaymentTerms::net_30(),
                performance: SupplierPerformance::new(Currency::usd()),
                credit_limit: None,
                current_balance: Money::new(rust_decimal_macros::dec!(0), Currency::usd()),
                tags: Vec::new(),
                notes: None,
                audit_info: AuditInfo::new(None),
            },
        }
    }

    /// Set supplier ID
    pub fn id(mut self, id: Uuid) -> Self {
        self.supplier.id = id;
        self
    }

    /// Set display name
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.supplier.display_name = Some(name.into());
        self
    }

    /// Set supplier type
    pub fn supplier_type(mut self, supplier_type: SupplierType) -> Self {
        self.supplier.supplier_type = supplier_type;
        self
    }

    /// Set status
    pub fn status(mut self, status: SupplierStatus) -> Self {
        self.supplier.status = status;
        self
    }

    /// Set contact information
    pub fn contact_info(mut self, contact_info: ContactInfo) -> Self {
        self.supplier.contact_info = contact_info;
        self
    }

    /// Set billing address
    pub fn billing_address(mut self, address: Address) -> Self {
        self.supplier.billing_address = Some(address);
        self
    }

    /// Set shipping address
    pub fn shipping_address(mut self, address: Address) -> Self {
        self.supplier.shipping_address = Some(address);
        self
    }

    /// Set currency
    pub fn currency(mut self, currency: Currency) -> Self {
        // Update currency in performance and balance
        self.supplier.performance.total_order_value.currency = currency.clone();
        self.supplier.current_balance.currency = currency.clone();
        self.supplier.currency = currency;
        self
    }

    /// Set payment terms
    pub fn payment_terms(mut self, terms: PaymentTerms) -> Self {
        self.supplier.payment_terms = terms;
        self
    }

    /// Set credit limit
    pub fn credit_limit(mut self, limit: Money) -> Self {
        self.supplier.credit_limit = Some(limit);
        self
    }

    /// Add tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.supplier.tags = tags;
        self
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.supplier.notes = Some(notes.into());
        self
    }

    /// Build the supplier
    pub fn build(self) -> Supplier {
        self.supplier
    }
}

impl Supplier {
    /// Create a new supplier builder
    pub fn builder(code: impl Into<String>, name: impl Into<String>) -> SupplierBuilder {
        SupplierBuilder::new(code, name)
    }

    /// Check if supplier is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, SupplierStatus::Active | SupplierStatus::Preferred)
    }

    /// Check if supplier can receive orders
    pub fn can_receive_orders(&self) -> bool {
        matches!(self.status, SupplierStatus::Active | SupplierStatus::Preferred)
    }

    /// Get effective shipping address (falls back to billing address)
    pub fn effective_shipping_address(&self) -> Option<&Address> {
        self.shipping_address.as_ref().or(self.billing_address.as_ref())
    }

    /// Check if supplier has exceeded credit limit
    pub fn is_over_credit_limit(&self) -> bool {
        if let Some(credit_limit) = &self.credit_limit {
            self.current_balance.amount > credit_limit.amount
        } else {
            false
        }
    }

    /// Get available credit
    pub fn available_credit(&self) -> Option<Money> {
        self.credit_limit.as_ref().map(|limit| {
            let available = limit.amount - self.current_balance.amount;
            Money::new(available.max(rust_decimal_macros::dec!(0)), limit.currency.clone())
        })
    }

    /// Check if supplier meets performance standards
    pub fn meets_performance_standards(&self) -> bool {
        use rust_decimal_macros::dec;
        self.performance.meets_criteria(dec!(3.0), dec!(85.0))
    }

    /// Update current balance
    pub fn update_balance(&mut self, amount: Money) -> Result<(), String> {
        if amount.currency != self.current_balance.currency {
            return Err("Currency mismatch".to_string());
        }
        self.current_balance.amount += amount.amount;
        Ok(())
    }

    /// Update audit info
    pub fn update_audit(&mut self, user_id: Option<String>) {
        self.audit_info.update(user_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_supplier_builder() {
        let supplier = Supplier::builder("SUP001", "Acme Corp")
            .supplier_type(SupplierType::Manufacturer)
            .status(SupplierStatus::Active)
            .currency(Currency::eur())
            .build();

        assert_eq!(supplier.code, "SUP001");
        assert_eq!(supplier.name, "Acme Corp");
        assert_eq!(supplier.supplier_type, SupplierType::Manufacturer);
        assert_eq!(supplier.status, SupplierStatus::Active);
        assert_eq!(supplier.currency.code, "EUR");
    }

    #[test]
    fn test_payment_terms_discount() {
        let terms = PaymentTerms::two_ten_net_30();
        let invoice = Money::new(dec!(1000.00), Currency::usd());
        
        let discount = terms.calculate_discount(&invoice).unwrap();
        assert_eq!(discount.amount, dec!(20.00)); // 2% of $1000
    }

    #[test]
    fn test_supplier_performance() {
        let mut performance = SupplierPerformance::new(Currency::usd());
        
        // Simulate some orders
        let order_value = Money::new(dec!(500.00), Currency::usd());
        performance.update_with_order(order_value.clone(), true, true); // Good order
        performance.update_with_order(order_value.clone(), false, true); // Late delivery
        performance.update_with_order(order_value, true, false); // Quality issue

        assert_eq!(performance.total_orders, 3);
        assert_eq!(performance.delivery_delays, 1);
        assert_eq!(performance.quality_issues, 1);
        
        // Check calculated percentages
        assert_eq!(performance.on_time_delivery_pct.unwrap(), dec!(66.67).round_dp(2));
        assert_eq!(performance.quality_acceptance_pct.unwrap(), dec!(66.67).round_dp(2));
    }

    #[test]
    fn test_credit_limit_checking() {
        let mut supplier = Supplier::builder("SUP002", "Test Supplier")
            .credit_limit(Money::new(dec!(10000.00), Currency::usd()))
            .build();

        assert!(!supplier.is_over_credit_limit());

        // Add some balance
        supplier.update_balance(Money::new(dec!(5000.00), Currency::usd())).unwrap();
        assert!(!supplier.is_over_credit_limit());

        // Exceed credit limit
        supplier.update_balance(Money::new(dec!(6000.00), Currency::usd())).unwrap();
        assert!(supplier.is_over_credit_limit());

        // Check available credit
        let available = supplier.available_credit().unwrap();
        assert_eq!(available.amount, dec!(0)); // Should be 0 when over limit
    }

    #[test]
    fn test_supplier_status_checks() {
        let active_supplier = Supplier::builder("SUP003", "Active Supplier")
            .status(SupplierStatus::Active)
            .build();

        assert!(active_supplier.is_active());
        assert!(active_supplier.can_receive_orders());

        let blocked_supplier = Supplier::builder("SUP004", "Blocked Supplier")
            .status(SupplierStatus::Blocked)
            .build();

        assert!(!blocked_supplier.is_active());
        assert!(!blocked_supplier.can_receive_orders());
    }
}