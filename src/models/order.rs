//! Order domain model
//!
//! This module defines the Order entity and related types for
//! managing purchase orders, sales orders, and order processing
//! workflows including approval, fulfillment, and tracking.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "schema")]
use schemars::JsonSchema;

use super::common::{Money, Currency, AuditInfo, UnitOfMeasure};
use super::warehouse::Address;

/// Order entity representing a purchase or sales order
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Order {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable order number
    pub order_number: String,
    /// Order type
    pub order_type: OrderType,
    /// Current status
    pub status: OrderStatus,
    /// Priority level
    pub priority: OrderPriority,
    
    // Parties
    /// Customer ID (for sales orders) or our company ID (for purchase orders)
    pub customer_id: Option<Uuid>,
    /// Supplier ID (for purchase orders)
    pub supplier_id: Option<Uuid>,
    /// Warehouse ID for fulfillment
    pub warehouse_id: Uuid,
    
    // Dates
    /// Date order was created
    pub order_date: DateTime<Utc>,
    /// Requested delivery date
    pub requested_delivery_date: Option<DateTime<Utc>>,
    /// Confirmed delivery date
    pub confirmed_delivery_date: Option<DateTime<Utc>>,
    /// Date order was shipped
    pub shipped_date: Option<DateTime<Utc>>,
    /// Date order was delivered
    pub delivered_date: Option<DateTime<Utc>>,
    
    // Financial Information
    /// Currency for all monetary values
    pub currency: Currency,
    /// Order line items
    pub line_items: Vec<OrderLineItem>,
    /// Subtotal (sum of line items)
    pub subtotal: Money,
    /// Tax amount
    pub tax_amount: Money,
    /// Shipping cost
    pub shipping_cost: Money,
    /// Discount amount
    pub discount_amount: Money,
    /// Total order amount
    pub total_amount: Money,
    
    // Addresses
    /// Billing address
    pub billing_address: Option<Address>,
    /// Shipping address
    pub shipping_address: Option<Address>,
    
    // Shipping Information
    /// Shipping method
    pub shipping_method: Option<String>,
    /// Tracking number
    pub tracking_number: Option<String>,
    /// Carrier name
    pub carrier: Option<String>,
    
    // References
    /// Purchase order number (for sales orders)
    pub customer_po_number: Option<String>,
    /// Reference to related orders
    pub related_orders: Vec<Uuid>,
    
    // Workflow
    /// Approval status
    pub approval_status: ApprovalStatus,
    /// Approved by user ID
    pub approved_by: Option<String>,
    /// Approval date
    pub approved_date: Option<DateTime<Utc>>,
    /// Fulfillment progress
    pub fulfillment: OrderFulfillment,
    
    // Metadata
    /// Special instructions
    pub special_instructions: Option<String>,
    /// Internal notes
    pub internal_notes: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom fields
    pub custom_fields: HashMap<String, String>,
    /// Audit information
    pub audit_info: AuditInfo,
}

/// Order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Purchase order (buying from supplier)
    Purchase,
    /// Sales order (selling to customer)
    Sales,
    /// Transfer order (between warehouses)
    Transfer,
    /// Return order
    Return,
    /// Adjustment order
    Adjustment,
}

/// Order status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Draft order, not yet submitted
    Draft,
    /// Submitted and waiting for approval
    Submitted,
    /// Approved and ready for processing
    Approved,
    /// Being picked/prepared
    Processing,
    /// Partially shipped
    PartiallyShipped,
    /// Fully shipped
    Shipped,
    /// Delivered to destination
    Delivered,
    /// Cancelled before fulfillment
    Cancelled,
    /// Returned after delivery
    Returned,
    /// On hold for various reasons
    OnHold,
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Order priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OrderPriority {
    Low,
    Normal,
    High,
    Urgent,
    Critical,
}

impl Default for OrderPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Approval status for orders
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// No approval required
    NotRequired,
    /// Waiting for approval
    Pending,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Needs additional approval
    RequiresAdditional,
}

impl Default for ApprovalStatus {
    fn default() -> Self {
        Self::NotRequired
    }
}

/// Order fulfillment tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct OrderFulfillment {
    /// Items picked so far
    pub picked_items: Vec<FulfilledLineItem>,
    /// Items packed so far
    pub packed_items: Vec<FulfilledLineItem>,
    /// Items shipped so far
    pub shipped_items: Vec<FulfilledLineItem>,
    /// Items delivered so far
    pub delivered_items: Vec<FulfilledLineItem>,
    
    /// Percentage complete (0-100)
    pub completion_percentage: Decimal,
    /// Estimated completion date
    pub estimated_completion: Option<DateTime<Utc>>,
    /// Fulfillment notes
    pub notes: Vec<FulfillmentNote>,
}

/// Fulfilled line item with tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct FulfilledLineItem {
    /// Original line item ID
    pub line_item_id: Uuid,
    /// Product ID
    pub product_id: Uuid,
    /// Quantity fulfilled
    pub quantity_fulfilled: Decimal,
    /// Date fulfilled
    pub fulfilled_date: DateTime<Utc>,
    /// Fulfilled by user
    pub fulfilled_by: Option<String>,
    /// Batch/serial numbers
    pub batch_numbers: Vec<String>,
    /// Location fulfilled from
    pub location: Option<String>,
}

/// Fulfillment note
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct FulfillmentNote {
    /// Note ID
    pub id: Uuid,
    /// Note content
    pub content: String,
    /// Note type
    pub note_type: FulfillmentNoteType,
    /// Created by user
    pub created_by: Option<String>,
    /// Creation date
    pub created_at: DateTime<Utc>,
}

/// Types of fulfillment notes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentNoteType {
    General,
    Issue,
    Delay,
    Quality,
    Shipping,
}

/// Individual line item within an order
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct OrderLineItem {
    /// Unique line item ID
    pub id: Uuid,
    /// Line number for display
    pub line_number: u32,
    /// Product ID being ordered
    pub product_id: Uuid,
    /// Product SKU at time of order
    pub product_sku: String,
    /// Product description at time of order
    pub product_description: String,
    
    /// Quantity ordered
    pub quantity_ordered: Decimal,
    /// Quantity shipped so far
    pub quantity_shipped: Decimal,
    /// Quantity delivered so far
    pub quantity_delivered: Decimal,
    /// Quantity returned
    pub quantity_returned: Decimal,
    
    /// Unit of measure
    pub unit_of_measure: UnitOfMeasure,
    /// Unit price at time of order
    pub unit_price: Money,
    /// Line discount percentage
    pub discount_percentage: Decimal,
    /// Line discount amount
    pub discount_amount: Money,
    /// Total line amount after discount
    pub line_total: Money,
    
    /// Tax rate for this line
    pub tax_rate: Decimal,
    /// Tax amount for this line
    pub tax_amount: Money,
    
    /// Requested delivery date for this line
    pub requested_delivery_date: Option<DateTime<Utc>>,
    /// Line status
    pub status: LineItemStatus,
    /// Special instructions for this line
    pub special_instructions: Option<String>,
}

/// Status of individual line items
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LineItemStatus {
    /// Pending processing
    Pending,
    /// Available for fulfillment
    Available,
    /// Back ordered (insufficient stock)
    BackOrdered,
    /// Being picked
    Picking,
    /// Picked and ready to ship
    Picked,
    /// Partially shipped
    PartiallyShipped,
    /// Fully shipped
    Shipped,
    /// Delivered
    Delivered,
    /// Cancelled
    Cancelled,
    /// Returned
    Returned,
}

impl Default for LineItemStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl OrderLineItem {
    /// Create new line item
    pub fn new(
        line_number: u32,
        product_id: Uuid,
        product_sku: String,
        product_description: String,
        quantity: Decimal,
        unit_price: Money,
        unit_of_measure: UnitOfMeasure,
    ) -> Self {
        use rust_decimal_macros::dec;
        
        let discount_percentage = dec!(0);
        let discount_amount = Money::new(dec!(0), unit_price.currency.clone());
        let line_total = Money::new(quantity * unit_price.amount, unit_price.currency.clone());
        let tax_rate = dec!(0);
        let tax_amount = Money::new(dec!(0), unit_price.currency.clone());
        
        Self {
            id: Uuid::new_v4(),
            line_number,
            product_id,
            product_sku,
            product_description,
            quantity_ordered: quantity,
            quantity_shipped: dec!(0),
            quantity_delivered: dec!(0),
            quantity_returned: dec!(0),
            unit_of_measure,
            unit_price,
            discount_percentage,
            discount_amount,
            line_total,
            tax_rate,
            tax_amount,
            requested_delivery_date: None,
            status: LineItemStatus::default(),
            special_instructions: None,
        }
    }

    /// Apply discount to line item
    pub fn apply_discount(&mut self, discount_percentage: Decimal) {
        use rust_decimal_macros::dec;
        
        self.discount_percentage = discount_percentage;
        let subtotal = self.quantity_ordered * self.unit_price.amount;
        self.discount_amount.amount = subtotal * (discount_percentage / dec!(100));
        self.line_total.amount = subtotal - self.discount_amount.amount;
        self.recalculate_tax();
    }

    /// Set tax rate and recalculate tax
    pub fn set_tax_rate(&mut self, tax_rate: Decimal) {
        self.tax_rate = tax_rate;
        self.recalculate_tax();
    }

    /// Recalculate tax amount
    fn recalculate_tax(&mut self) {
        use rust_decimal_macros::dec;
        self.tax_amount.amount = self.line_total.amount * (self.tax_rate / dec!(100));
    }

    /// Get remaining quantity to ship
    pub fn remaining_to_ship(&self) -> Decimal {
        (self.quantity_ordered - self.quantity_shipped).max(rust_decimal_macros::dec!(0))
    }

    /// Get remaining quantity to deliver
    pub fn remaining_to_deliver(&self) -> Decimal {
        (self.quantity_shipped - self.quantity_delivered).max(rust_decimal_macros::dec!(0))
    }

    /// Check if line is fully shipped
    pub fn is_fully_shipped(&self) -> bool {
        self.quantity_shipped >= self.quantity_ordered
    }

    /// Check if line is fully delivered
    pub fn is_fully_delivered(&self) -> bool {
        self.quantity_delivered >= self.quantity_ordered
    }

    /// Record shipment
    pub fn record_shipment(&mut self, quantity: Decimal) -> Result<(), String> {
        if quantity <= rust_decimal_macros::dec!(0) {
            return Err("Quantity must be positive".to_string());
        }
        
        let remaining = self.remaining_to_ship();
        if quantity > remaining {
            return Err(format!("Cannot ship {} units, only {} remaining", quantity, remaining));
        }
        
        self.quantity_shipped += quantity;
        
        // Update status
        if self.is_fully_shipped() {
            self.status = LineItemStatus::Shipped;
        } else {
            self.status = LineItemStatus::PartiallyShipped;
        }
        
        Ok(())
    }

    /// Record delivery
    pub fn record_delivery(&mut self, quantity: Decimal) -> Result<(), String> {
        if quantity <= rust_decimal_macros::dec!(0) {
            return Err("Quantity must be positive".to_string());
        }
        
        let remaining = self.remaining_to_deliver();
        if quantity > remaining {
            return Err(format!("Cannot deliver {} units, only {} remaining", quantity, remaining));
        }
        
        self.quantity_delivered += quantity;
        
        // Update status
        if self.is_fully_delivered() {
            self.status = LineItemStatus::Delivered;
        }
        
        Ok(())
    }
}

impl OrderFulfillment {
    /// Create new fulfillment tracker
    pub fn new() -> Self {
        Self {
            picked_items: Vec::new(),
            packed_items: Vec::new(),
            shipped_items: Vec::new(),
            delivered_items: Vec::new(),
            completion_percentage: rust_decimal_macros::dec!(0),
            estimated_completion: None,
            notes: Vec::new(),
        }
    }

    /// Add fulfillment note
    pub fn add_note(&mut self, content: String, note_type: FulfillmentNoteType, user_id: Option<String>) {
        self.notes.push(FulfillmentNote {
            id: Uuid::new_v4(),
            content,
            note_type,
            created_by: user_id,
            created_at: Utc::now(),
        });
    }

    /// Record item picked
    pub fn record_picked(&mut self, line_item_id: Uuid, product_id: Uuid, quantity: Decimal, user_id: Option<String>) {
        self.picked_items.push(FulfilledLineItem {
            line_item_id,
            product_id,
            quantity_fulfilled: quantity,
            fulfilled_date: Utc::now(),
            fulfilled_by: user_id,
            batch_numbers: Vec::new(),
            location: None,
        });
    }

    /// Calculate completion percentage based on line items
    pub fn calculate_completion(&mut self, line_items: &[OrderLineItem]) {
        if line_items.is_empty() {
            self.completion_percentage = rust_decimal_macros::dec!(100);
            return;
        }

        let total_quantity: Decimal = line_items.iter()
            .map(|item| item.quantity_ordered)
            .sum();
        
        let shipped_quantity: Decimal = line_items.iter()
            .map(|item| item.quantity_shipped)
            .sum();
        
        if total_quantity > rust_decimal_macros::dec!(0) {
            self.completion_percentage = (shipped_quantity / total_quantity) * rust_decimal_macros::dec!(100);
        } else {
            self.completion_percentage = rust_decimal_macros::dec!(100);
        }
    }
}

impl Default for OrderFulfillment {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating orders
pub struct OrderBuilder {
    order: Order,
}

impl OrderBuilder {
    /// Create new order builder
    pub fn new(order_number: String, order_type: OrderType, warehouse_id: Uuid, currency: Currency) -> Self {
        Self {
            order: Order {
                id: Uuid::new_v4(),
                order_number,
                order_type,
                status: OrderStatus::default(),
                priority: OrderPriority::default(),
                customer_id: None,
                supplier_id: None,
                warehouse_id,
                order_date: Utc::now(),
                requested_delivery_date: None,
                confirmed_delivery_date: None,
                shipped_date: None,
                delivered_date: None,
                currency: currency.clone(),
                line_items: Vec::new(),
                subtotal: Money::new(rust_decimal_macros::dec!(0), currency.clone()),
                tax_amount: Money::new(rust_decimal_macros::dec!(0), currency.clone()),
                shipping_cost: Money::new(rust_decimal_macros::dec!(0), currency.clone()),
                discount_amount: Money::new(rust_decimal_macros::dec!(0), currency.clone()),
                total_amount: Money::new(rust_decimal_macros::dec!(0), currency),
                billing_address: None,
                shipping_address: None,
                shipping_method: None,
                tracking_number: None,
                carrier: None,
                customer_po_number: None,
                related_orders: Vec::new(),
                approval_status: ApprovalStatus::default(),
                approved_by: None,
                approved_date: None,
                fulfillment: OrderFulfillment::default(),
                special_instructions: None,
                internal_notes: None,
                tags: Vec::new(),
                custom_fields: HashMap::new(),
                audit_info: AuditInfo::new(None),
            },
        }
    }

    /// Set order ID
    pub fn id(mut self, id: Uuid) -> Self {
        self.order.id = id;
        self
    }

    /// Set customer ID
    pub fn customer_id(mut self, customer_id: Uuid) -> Self {
        self.order.customer_id = Some(customer_id);
        self
    }

    /// Set supplier ID
    pub fn supplier_id(mut self, supplier_id: Uuid) -> Self {
        self.order.supplier_id = Some(supplier_id);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: OrderPriority) -> Self {
        self.order.priority = priority;
        self
    }

    /// Set requested delivery date
    pub fn requested_delivery_date(mut self, date: DateTime<Utc>) -> Self {
        self.order.requested_delivery_date = Some(date);
        self
    }

    /// Add line item
    pub fn add_line_item(mut self, line_item: OrderLineItem) -> Self {
        self.order.line_items.push(line_item);
        self
    }

    /// Set billing address
    pub fn billing_address(mut self, address: Address) -> Self {
        self.order.billing_address = Some(address);
        self
    }

    /// Set shipping address
    pub fn shipping_address(mut self, address: Address) -> Self {
        self.order.shipping_address = Some(address);
        self
    }

    /// Set customer PO number
    pub fn customer_po_number(mut self, po_number: String) -> Self {
        self.order.customer_po_number = Some(po_number);
        self
    }

    /// Set special instructions
    pub fn special_instructions(mut self, instructions: String) -> Self {
        self.order.special_instructions = Some(instructions);
        self
    }

    /// Build the order and calculate totals
    pub fn build(mut self) -> Order {
        self.order.calculate_totals();
        self.order
    }
}

impl Order {
    /// Create new order builder
    pub fn builder(order_number: String, order_type: OrderType, warehouse_id: Uuid, currency: Currency) -> OrderBuilder {
        OrderBuilder::new(order_number, order_type, warehouse_id, currency)
    }

    /// Calculate order totals
    pub fn calculate_totals(&mut self) {
        use rust_decimal_macros::dec;
        
        // Calculate subtotal
        self.subtotal.amount = self.line_items.iter()
            .map(|item| item.line_total.amount)
            .sum();
        
        // Calculate total tax
        self.tax_amount.amount = self.line_items.iter()
            .map(|item| item.tax_amount.amount)
            .sum();
        
        // Calculate total amount
        self.total_amount.amount = self.subtotal.amount + self.tax_amount.amount 
            + self.shipping_cost.amount - self.discount_amount.amount;
    }

    /// Add line item to order
    pub fn add_line_item(&mut self, line_item: OrderLineItem) {
        self.line_items.push(line_item);
        self.calculate_totals();
    }

    /// Remove line item by ID
    pub fn remove_line_item(&mut self, line_item_id: Uuid) -> bool {
        let original_len = self.line_items.len();
        self.line_items.retain(|item| item.id != line_item_id);
        let removed = self.line_items.len() < original_len;
        if removed {
            self.calculate_totals();
        }
        removed
    }

    /// Get line item by ID
    pub fn get_line_item(&self, line_item_id: Uuid) -> Option<&OrderLineItem> {
        self.line_items.iter().find(|item| item.id == line_item_id)
    }

    /// Get mutable line item by ID
    pub fn get_line_item_mut(&mut self, line_item_id: Uuid) -> Option<&mut OrderLineItem> {
        self.line_items.iter_mut().find(|item| item.id == line_item_id)
    }

    /// Check if order can be cancelled
    pub fn can_be_cancelled(&self) -> bool {
        matches!(self.status, OrderStatus::Draft | OrderStatus::Submitted | OrderStatus::Approved)
    }

    /// Check if order requires approval
    pub fn requires_approval(&self) -> bool {
        matches!(self.approval_status, ApprovalStatus::Pending | ApprovalStatus::RequiresAdditional)
    }

    /// Approve order
    pub fn approve(&mut self, approved_by: String) -> Result<(), String> {
        if !matches!(self.approval_status, ApprovalStatus::Pending | ApprovalStatus::RequiresAdditional) {
            return Err("Order is not pending approval".to_string());
        }
        
        self.approval_status = ApprovalStatus::Approved;
        self.approved_by = Some(approved_by);
        self.approved_date = Some(Utc::now());
        
        if matches!(self.status, OrderStatus::Submitted) {
            self.status = OrderStatus::Approved;
        }
        
        Ok(())
    }

    /// Reject order
    pub fn reject(&mut self) -> Result<(), String> {
        if !matches!(self.approval_status, ApprovalStatus::Pending | ApprovalStatus::RequiresAdditional) {
            return Err("Order is not pending approval".to_string());
        }
        
        self.approval_status = ApprovalStatus::Rejected;
        Ok(())
    }

    /// Submit order for processing
    pub fn submit(&mut self) -> Result<(), String> {
        if !matches!(self.status, OrderStatus::Draft) {
            return Err("Only draft orders can be submitted".to_string());
        }
        
        if self.line_items.is_empty() {
            return Err("Cannot submit order without line items".to_string());
        }
        
        self.status = OrderStatus::Submitted;
        
        // Set approval status based on order value or other criteria
        if self.total_amount.amount > rust_decimal_macros::dec!(10000) {
            self.approval_status = ApprovalStatus::Pending;
        } else {
            self.approval_status = ApprovalStatus::NotRequired;
            self.status = OrderStatus::Approved;
        }
        
        Ok(())
    }

    /// Calculate fulfillment percentage
    pub fn calculate_fulfillment_percentage(&self) -> Decimal {
        if self.line_items.is_empty() {
            return rust_decimal_macros::dec!(100);
        }
        
        let total_quantity: Decimal = self.line_items.iter()
            .map(|item| item.quantity_ordered)
            .sum();
        
        let shipped_quantity: Decimal = self.line_items.iter()
            .map(|item| item.quantity_shipped)
            .sum();
        
        if total_quantity > rust_decimal_macros::dec!(0) {
            (shipped_quantity / total_quantity) * rust_decimal_macros::dec!(100)
        } else {
            rust_decimal_macros::dec!(100)
        }
    }

    /// Check if order is fully fulfilled
    pub fn is_fully_fulfilled(&self) -> bool {
        self.line_items.iter().all(|item| item.is_fully_shipped())
    }

    /// Get effective shipping address (falls back to billing address)
    pub fn effective_shipping_address(&self) -> Option<&Address> {
        self.shipping_address.as_ref().or(self.billing_address.as_ref())
    }

    /// Update audit info
    pub fn update_audit(&mut self, user_id: Option<String>) {
        self.audit_info.update(user_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::common::{Currency, UnitOfMeasure};
    use rust_decimal_macros::dec;

    fn create_test_line_item() -> OrderLineItem {
        OrderLineItem::new(
            1,
            Uuid::new_v4(),
            "TEST-001".to_string(),
            "Test Product".to_string(),
            dec!(10),
            Money::new(dec!(15.00), Currency::usd()),
            UnitOfMeasure::each(),
        )
    }

    #[test]
    fn test_order_builder() {
        let warehouse_id = Uuid::new_v4();
        let order = Order::builder(
            "PO-001".to_string(),
            OrderType::Purchase,
            warehouse_id,
            Currency::usd(),
        )
        .priority(OrderPriority::High)
        .add_line_item(create_test_line_item())
        .build();

        assert_eq!(order.order_number, "PO-001");
        assert_eq!(order.order_type, OrderType::Purchase);
        assert_eq!(order.priority, OrderPriority::High);
        assert_eq!(order.warehouse_id, warehouse_id);
        assert_eq!(order.line_items.len(), 1);
        assert_eq!(order.subtotal.amount, dec!(150.00)); // 10 * $15
    }

    #[test]
    fn test_line_item_discount() {
        let mut line_item = create_test_line_item();
        line_item.apply_discount(dec!(10)); // 10% discount

        assert_eq!(line_item.discount_percentage, dec!(10));
        assert_eq!(line_item.discount_amount.amount, dec!(15.00)); // 10% of $150
        assert_eq!(line_item.line_total.amount, dec!(135.00)); // $150 - $15
    }

    #[test]
    fn test_line_item_tax() {
        let mut line_item = create_test_line_item();
        line_item.set_tax_rate(dec!(8.5)); // 8.5% tax

        assert_eq!(line_item.tax_rate, dec!(8.5));
        assert_eq!(line_item.tax_amount.amount, dec!(12.75)); // 8.5% of $150
    }

    #[test]
    fn test_line_item_shipment() {
        let mut line_item = create_test_line_item();
        
        // Ship partial quantity
        line_item.record_shipment(dec!(6)).unwrap();
        assert_eq!(line_item.quantity_shipped, dec!(6));
        assert_eq!(line_item.status, LineItemStatus::PartiallyShipped);
        assert_eq!(line_item.remaining_to_ship(), dec!(4));
        
        // Ship remaining quantity
        line_item.record_shipment(dec!(4)).unwrap();
        assert_eq!(line_item.quantity_shipped, dec!(10));
        assert_eq!(line_item.status, LineItemStatus::Shipped);
        assert!(line_item.is_fully_shipped());
    }

    #[test]
    fn test_order_submission_and_approval() {
        let mut order = Order::builder(
            "PO-002".to_string(),
            OrderType::Purchase,
            Uuid::new_v4(),
            Currency::usd(),
        )
        .add_line_item(create_test_line_item())
        .build();

        // Submit order
        order.submit().unwrap();
        assert_eq!(order.status, OrderStatus::Approved); // Auto-approved for small orders
        assert_eq!(order.approval_status, ApprovalStatus::NotRequired);
    }

    #[test]
    fn test_large_order_requires_approval() {
        let mut large_line_item = create_test_line_item();
        large_line_item.unit_price = Money::new(dec!(1500.00), Currency::usd()); // $15,000 total
        
        let mut order = Order::builder(
            "PO-003".to_string(),
            OrderType::Purchase,
            Uuid::new_v4(),
            Currency::usd(),
        )
        .add_line_item(large_line_item)
        .build();

        // Submit order
        order.submit().unwrap();
        assert_eq!(order.status, OrderStatus::Submitted);
        assert_eq!(order.approval_status, ApprovalStatus::Pending);

        // Approve order
        order.approve("manager@example.com".to_string()).unwrap();
        assert_eq!(order.approval_status, ApprovalStatus::Approved);
        assert!(order.approved_by.is_some());
        assert!(order.approved_date.is_some());
    }

    #[test]
    fn test_order_fulfillment_percentage() {
        let mut order = Order::builder(
            "PO-004".to_string(),
            OrderType::Purchase,
            Uuid::new_v4(),
            Currency::usd(),
        )
        .add_line_item(create_test_line_item())
        .build();

        assert_eq!(order.calculate_fulfillment_percentage(), dec!(0)); // Nothing shipped yet

        // Ship partial quantity
        if let Some(line_item) = order.line_items.get_mut(0) {
            line_item.record_shipment(dec!(5)).unwrap();
        }
        assert_eq!(order.calculate_fulfillment_percentage(), dec!(50)); // 50% shipped
    }

    #[test]
    fn test_order_cancellation() {
        let order = Order::builder(
            "PO-005".to_string(),
            OrderType::Purchase,
            Uuid::new_v4(),
            Currency::usd(),
        )
        .build();

        assert!(order.can_be_cancelled()); // Draft orders can be cancelled

        let mut shipped_order = order;
        shipped_order.status = OrderStatus::Shipped;
        assert!(!shipped_order.can_be_cancelled()); // Shipped orders cannot be cancelled
    }
}