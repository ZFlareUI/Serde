//! # Inventory Management Framework
//!
//! A modular, production-grade inventory management framework for Rust developers
//! building logistics, e-commerce, or ERP systems. Features async operations,
//! multiple database backends, REST/GraphQL APIs, and ML-based forecasting.
//!
//! ## Features
//!
//! ### Core Features
//! - **Advanced Serde Integration**: Custom serialization for complex business objects
//! - **Multi-format Support**: JSON, TOML, CSV with automatic format detection
//! - **Business Logic**: Economic Order Quantity (EOQ), ABC analysis, demand forecasting
//! - **Type Safety**: Builder patterns with compile-time validation
//! - **Data Pipelines**: Functional transformation and analytics chains
//! - **Production Ready**: Comprehensive error handling and logging
//!
//! ### Enterprise Features
//! - **Machine Learning & Predictive Analytics**: ARIMA forecasting, neural networks, ensemble models
//! - **Multi-Warehouse Optimization**: Network flow algorithms, genetic optimization, supply chain routing
//! - **Advanced Inventory Strategies**: JIT, VMI, two-echelon policies with dynamic safety stock
//! - **Financial Optimization**: FIFO/LIFO/weighted average costing, transfer pricing, tax optimization
//! - **Quality Control**: Statistical process control, supplier quality metrics, defect tracking
//! - **Real-Time Decision Support**: Event streaming, alerting, recommendation engines, simulations
//!
//! ## Quick Start
//!
//! ```rust
//! use inventory_serde::prelude::*;
//! use inventory_serde::models::{product::{Pricing, InventoryLevels}, common::Currency};
//! use rust_decimal_macros::dec;
//!
//! // Create a product using the builder pattern
//! let pricing = Pricing::new(dec!(25.50), dec!(39.99), "USD");
//! let inventory = InventoryLevels::new(100, 10, 500, 50);
//! 
//! let product = ProductBuilder::new()
//!     .sku("SKU-001".to_string())
//!     .name("Industrial Widget".to_string())
//!     .description("High-quality widget for industrial applications".to_string())
//!     .category(ProductCategory::Industrial)
//!     .pricing(pricing)
//!     .inventory(inventory)
//!     .build()?;
//!
//! // Serialize to JSON
//! let json = serde_json::to_string_pretty(&product)?;
//! println!("Product: {}", product.name);
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Core modules
pub mod models;
pub mod repository;

// Feature-gated modules
#[cfg(feature = "services")]
pub mod services;

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "utils")]
pub mod utils;

// Re-export common types for convenience
pub use models::{
    Product, ProductBuilder, ProductCategory, ProductStatus,
    Warehouse, WarehouseBuilder, WarehouseType, WarehouseStatus,
    StockTransaction, StockTransactionBuilder, TransactionType, TransactionStatus,
};
pub use models::supplier::{Supplier, SupplierBuilder, SupplierType, SupplierStatus};
pub use models::order::{Order, OrderBuilder, OrderType, OrderStatus};

pub use repository::{
    Repository, SearchableRepository, RepositoryResult, RepositoryError,
    ProductRepository, WarehouseRepository, SupplierRepository, 
    OrderRepository, StockTransactionRepository,
    memory::MemoryRepositoryFactory,
};

// Re-export external dependencies for convenience
pub use chrono::{DateTime, Utc};
pub use rust_decimal::Decimal;
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;

// Feature-specific re-exports
#[cfg(feature = "tokio-runtime")]
pub use tokio;

#[cfg(feature = "sql")]
pub use sqlx;

#[cfg(feature = "mongodb")]
pub use mongodb;

#[cfg(feature = "api")]
pub use axum;

#[cfg(feature = "schema")]
pub use schemars;

/// Convenience module for common imports
pub mod prelude {
    pub use crate::models::{
        Product, ProductBuilder, ProductCategory, ProductStatus,
        Warehouse, WarehouseBuilder, WarehouseType, WarehouseStatus,
        StockTransaction, StockTransactionBuilder, TransactionType, TransactionStatus,
    };
    pub use crate::models::supplier::{Supplier, SupplierBuilder, SupplierType, SupplierStatus};
    pub use crate::models::order::{Order, OrderBuilder, OrderType, OrderStatus};
    
    pub use crate::repository::{
        Repository, SearchableRepository, RepositoryResult, RepositoryError,
        ProductRepository, WarehouseRepository, SupplierRepository,
        OrderRepository, StockTransactionRepository,
        memory::MemoryRepositoryFactory,
    };
    
    pub use chrono::{DateTime, Utc};
    pub use rust_decimal::Decimal;
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;
}