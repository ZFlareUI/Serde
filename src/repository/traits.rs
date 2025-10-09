//! Repository trait definitions
//!
//! This module defines all the specific repository traits for each domain entity
//! with their specialized methods and filter types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{
    Product, ProductCategory, ProductStatus,
    Warehouse, WarehouseType, WarehouseStatus,
    StockTransaction, TransactionType, TransactionStatus,
};
use crate::models::supplier::{Supplier, SupplierType, SupplierStatus};
use crate::models::order::{Order, OrderType, OrderStatus, OrderPriority};

use super::{SearchableRepository, RepositoryResult};

/// Product repository trait
#[async_trait]
pub trait ProductRepository: SearchableRepository<Product, ProductFilters> + Send + Sync {
    /// Get product by SKU
    async fn get_by_sku(&self, sku: &str) -> RepositoryResult<Option<Product>>;
    
    /// Get products by category
    async fn get_by_category(&self, category: ProductCategory) -> RepositoryResult<Vec<Product>>;
    
    /// Get products with low inventory
    async fn get_low_inventory(&self, warehouse_id: Option<Uuid>) -> RepositoryResult<Vec<Product>>;
    
    /// Update inventory levels
    async fn update_inventory_levels(&self, product_id: Uuid, warehouse_id: Uuid, quantity_change: Decimal) -> RepositoryResult<()>;
    
    /// Get products by supplier
    async fn get_by_supplier(&self, supplier_id: Uuid) -> RepositoryResult<Vec<Product>>;
    
    /// Bulk update prices
    async fn bulk_update_prices(&self, price_updates: HashMap<Uuid, Decimal>) -> RepositoryResult<()>;
}

/// Product search filters
#[derive(Debug, Clone, Default)]
pub struct ProductFilters {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub category: Option<ProductCategory>,
    pub status: Option<ProductStatus>,
    pub supplier_id: Option<Uuid>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub low_inventory: Option<bool>,
    pub tags: Vec<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// Warehouse repository trait
#[async_trait]
pub trait WarehouseRepository: SearchableRepository<Warehouse, WarehouseFilters> + Send + Sync {
    /// Get warehouse by code
    async fn get_by_code(&self, code: &str) -> RepositoryResult<Option<Warehouse>>;
    
    /// Get warehouses by type
    async fn get_by_type(&self, warehouse_type: WarehouseType) -> RepositoryResult<Vec<Warehouse>>;
    
    /// Get warehouses near location
    async fn get_near_location(&self, latitude: Decimal, longitude: Decimal, radius_km: Decimal) -> RepositoryResult<Vec<Warehouse>>;
    
    /// Get warehouse utilization stats
    async fn get_utilization_stats(&self, warehouse_id: Uuid) -> RepositoryResult<WarehouseUtilization>;
    
    /// Update capacity
    async fn update_capacity(&self, warehouse_id: Uuid, used_capacity: Decimal) -> RepositoryResult<()>;
}

/// Warehouse utilization statistics
#[derive(Debug, Clone)]
pub struct WarehouseUtilization {
    pub warehouse_id: Uuid,
    pub total_capacity: Decimal,
    pub used_capacity: Decimal,
    pub available_capacity: Decimal,
    pub utilization_percentage: Decimal,
    pub product_count: u64,
    pub last_updated: DateTime<Utc>,
}

/// Warehouse search filters
#[derive(Debug, Clone, Default)]
pub struct WarehouseFilters {
    pub code: Option<String>,
    pub name: Option<String>,
    pub warehouse_type: Option<WarehouseType>,
    pub status: Option<WarehouseStatus>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub min_capacity: Option<Decimal>,
    pub max_capacity: Option<Decimal>,
    pub latitude_range: Option<(Decimal, Decimal)>,
    pub longitude_range: Option<(Decimal, Decimal)>,
}

/// Supplier repository trait
#[async_trait]
pub trait SupplierRepository: SearchableRepository<Supplier, SupplierFilters> + Send + Sync {
    /// Get supplier by code
    async fn get_by_code(&self, code: &str) -> RepositoryResult<Option<Supplier>>;
    
    /// Get suppliers by type
    async fn get_by_type(&self, supplier_type: SupplierType) -> RepositoryResult<Vec<Supplier>>;
    
    /// Get preferred suppliers
    async fn get_preferred(&self) -> RepositoryResult<Vec<Supplier>>;
    
    /// Update performance metrics
    async fn update_performance(&self, supplier_id: Uuid, order_value: Decimal, on_time: bool, quality_ok: bool) -> RepositoryResult<()>;
    
    /// Update balance
    async fn update_balance(&self, supplier_id: Uuid, amount_change: Decimal) -> RepositoryResult<()>;
    
    /// Get suppliers over credit limit
    async fn get_over_credit_limit(&self) -> RepositoryResult<Vec<Supplier>>;
}

/// Supplier search filters
#[derive(Debug, Clone, Default)]
pub struct SupplierFilters {
    pub code: Option<String>,
    pub name: Option<String>,
    pub supplier_type: Option<SupplierType>,
    pub status: Option<SupplierStatus>,
    pub country: Option<String>,
    pub min_rating: Option<Decimal>,
    pub max_balance: Option<Decimal>,
    pub over_credit_limit: Option<bool>,
    pub tags: Vec<String>,
}

/// Order repository trait
#[async_trait]
pub trait OrderRepository: SearchableRepository<Order, OrderFilters> + Send + Sync {
    /// Get order by number
    async fn get_by_number(&self, order_number: &str) -> RepositoryResult<Option<Order>>;
    
    /// Get orders by customer
    async fn get_by_customer(&self, customer_id: Uuid) -> RepositoryResult<Vec<Order>>;
    
    /// Get orders by supplier
    async fn get_by_supplier(&self, supplier_id: Uuid) -> RepositoryResult<Vec<Order>>;
    
    /// Get orders by warehouse
    async fn get_by_warehouse(&self, warehouse_id: Uuid) -> RepositoryResult<Vec<Order>>;
    
    /// Get orders requiring approval
    async fn get_pending_approval(&self) -> RepositoryResult<Vec<Order>>;
    
    /// Get overdue orders
    async fn get_overdue(&self) -> RepositoryResult<Vec<Order>>;
    
    /// Update order status
    async fn update_status(&self, order_id: Uuid, status: OrderStatus) -> RepositoryResult<()>;
    
    /// Record shipment for line item
    async fn record_line_item_shipment(&self, order_id: Uuid, line_item_id: Uuid, quantity: Decimal) -> RepositoryResult<()>;
    
    /// Get order analytics
    async fn get_analytics(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> RepositoryResult<OrderAnalytics>;
}

/// Order analytics data
#[derive(Debug, Clone)]
pub struct OrderAnalytics {
    pub total_orders: u64,
    pub total_value: Decimal,
    pub average_order_value: Decimal,
    pub orders_by_status: HashMap<OrderStatus, u64>,
    pub orders_by_type: HashMap<OrderType, u64>,
    pub top_customers: Vec<(Uuid, u64)>, // (customer_id, order_count)
    pub fulfillment_rate: Decimal,
    pub average_processing_time_hours: Decimal,
}

/// Order search filters
#[derive(Debug, Clone, Default)]
pub struct OrderFilters {
    pub order_number: Option<String>,
    pub order_type: Option<OrderType>,
    pub status: Option<OrderStatus>,
    pub priority: Option<OrderPriority>,
    pub customer_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub min_total: Option<Decimal>,
    pub max_total: Option<Decimal>,
    pub order_date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub delivery_date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub requires_approval: Option<bool>,
    pub overdue: Option<bool>,
    pub tags: Vec<String>,
}

/// Stock transaction repository trait
#[async_trait]
pub trait StockTransactionRepository: SearchableRepository<StockTransaction, TransactionFilters> + Send + Sync {
    /// Get transactions by product
    async fn get_by_product(&self, product_id: Uuid) -> RepositoryResult<Vec<StockTransaction>>;
    
    /// Get transactions by warehouse
    async fn get_by_warehouse(&self, warehouse_id: Uuid) -> RepositoryResult<Vec<StockTransaction>>;
    
    /// Get transactions by reference
    async fn get_by_reference(&self, reference_type: &str, reference_id: Uuid) -> RepositoryResult<Vec<StockTransaction>>;
    
    /// Calculate inventory balance
    async fn calculate_balance(&self, product_id: Uuid, warehouse_id: Uuid, as_of_date: Option<DateTime<Utc>>) -> RepositoryResult<Decimal>;
    
    /// Get inventory movements
    async fn get_movements(&self, product_id: Uuid, warehouse_id: Uuid, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> RepositoryResult<Vec<StockTransaction>>;
    
    /// Create adjustment transaction
    async fn create_adjustment(&self, product_id: Uuid, warehouse_id: Uuid, quantity: Decimal, reason: &str, user_id: Option<String>) -> RepositoryResult<StockTransaction>;
    
    /// Get inventory valuation
    async fn get_inventory_valuation(&self, warehouse_id: Option<Uuid>, as_of_date: Option<DateTime<Utc>>) -> RepositoryResult<InventoryValuation>;
}

/// Inventory valuation data
#[derive(Debug, Clone)]
pub struct InventoryValuation {
    pub total_quantity: Decimal,
    pub total_cost: Decimal,
    pub total_value: Decimal,
    pub items: Vec<InventoryItem>,
    pub as_of_date: DateTime<Utc>,
}

/// Individual inventory item in valuation
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub quantity: Decimal,
    pub unit_cost: Decimal,
    pub total_cost: Decimal,
    pub current_market_value: Option<Decimal>,
}

/// Stock transaction search filters
#[derive(Debug, Clone, Default)]
pub struct TransactionFilters {
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub product_id: Option<Uuid>,
    pub warehouse_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub min_quantity: Option<Decimal>,
    pub max_quantity: Option<Decimal>,
    pub user_id: Option<String>,
    pub batch_number: Option<String>,
}

/// Database transaction trait for atomic operations
#[async_trait]
pub trait DatabaseTransaction: Send + Sync {
    /// Commit the transaction
    async fn commit(self: Box<Self>) -> RepositoryResult<()>;
    
    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> RepositoryResult<()>;
}

/// Unit of Work pattern for managing multiple repository operations
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type Transaction: DatabaseTransaction;
    
    /// Begin a new transaction
    async fn begin(&self) -> RepositoryResult<Self::Transaction>;
    
    /// Execute multiple operations within a transaction
    async fn execute<F, T>(&self, operation: F) -> RepositoryResult<T>
    where
        F: FnOnce() -> RepositoryResult<T> + Send + 'static,
        T: Send + 'static;
}

/// Repository factory trait for creating repository instances
pub trait RepositoryFactory: Send + Sync {
    type ProductRepo: ProductRepository;
    type WarehouseRepo: WarehouseRepository;
    type SupplierRepo: SupplierRepository;
    type OrderRepo: OrderRepository;
    type TransactionRepo: StockTransactionRepository;
    
    /// Get product repository
    fn product_repository(&self) -> &Self::ProductRepo;
    
    /// Get warehouse repository
    fn warehouse_repository(&self) -> &Self::WarehouseRepo;
    
    /// Get supplier repository
    fn supplier_repository(&self) -> &Self::SupplierRepo;
    
    /// Get order repository
    fn order_repository(&self) -> &Self::OrderRepo;
    
    /// Get transaction repository
    fn transaction_repository(&self) -> &Self::TransactionRepo;
}

/// Configuration for repository connections
#[derive(Debug, Clone)]
pub struct RepositoryConfig {
    /// Database connection string
    pub connection_string: String,
    
    /// Maximum number of connections in pool
    pub max_connections: u32,
    
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    
    /// Query timeout in seconds
    pub query_timeout_seconds: u64,
    
    /// Enable query logging
    pub enable_logging: bool,
    
    /// Additional configuration options
    pub options: HashMap<String, String>,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            connection_string: "sqlite::memory:".to_string(),
            max_connections: 10,
            connection_timeout_seconds: 30,
            query_timeout_seconds: 60,
            enable_logging: false,
            options: HashMap::new(),
        }
    }
}

impl RepositoryConfig {
    /// Create config for PostgreSQL
    pub fn postgres(database_url: impl Into<String>) -> Self {
        Self {
            connection_string: database_url.into(),
            max_connections: 20,
            connection_timeout_seconds: 30,
            query_timeout_seconds: 60,
            enable_logging: true,
            options: HashMap::from([
                ("application_name".to_string(), "inventory-serde".to_string()),
                ("sslmode".to_string(), "prefer".to_string()),
            ]),
        }
    }

    /// Create config for MySQL
    pub fn mysql(database_url: impl Into<String>) -> Self {
        Self {
            connection_string: database_url.into(),
            max_connections: 15,
            connection_timeout_seconds: 30,
            query_timeout_seconds: 60,
            enable_logging: true,
            options: HashMap::new(),
        }
    }

    /// Create config for SQLite
    pub fn sqlite(database_path: impl Into<String>) -> Self {
        Self {
            connection_string: format!("sqlite:{}", database_path.into()),
            max_connections: 1, // SQLite doesn't support multiple writers
            connection_timeout_seconds: 30,
            query_timeout_seconds: 60,
            enable_logging: false,
            options: HashMap::from([
                ("journal_mode".to_string(), "WAL".to_string()),
                ("synchronous".to_string(), "NORMAL".to_string()),
            ]),
        }
    }

    /// Create config for MongoDB
    pub fn mongodb(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            max_connections: 10,
            connection_timeout_seconds: 30,
            query_timeout_seconds: 60,
            enable_logging: true,
            options: HashMap::from([
                ("appName".to_string(), "inventory-serde".to_string()),
                ("retryWrites".to_string(), "true".to_string()),
            ]),
        }
    }

    /// Create in-memory config for testing
    pub fn memory() -> Self {
        Self {
            connection_string: "memory://test".to_string(),
            max_connections: 1,
            connection_timeout_seconds: 1,
            query_timeout_seconds: 10,
            enable_logging: false,
            options: HashMap::new(),
        }
    }
}