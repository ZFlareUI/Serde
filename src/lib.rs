//! # Advanced Enterprise Inventory Management Library
//!
//! A comprehensive, production-ready Rust library for enterprise inventory management
//! with advanced serde serialization capabilities, machine learning forecasting,
//! multi-warehouse optimization, and real-time decision support.
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
//! use serde_inventory_lib::prelude::*;
//! use chrono::Utc;
//! use uuid::Uuid;
//!
//! // Create a product using the builder pattern
//! let product = ProductBuilder::new()
//!     .name("Industrial Widget")
//!     .description("High-quality widget for industrial applications")
//!     .category("Manufacturing")
//!     .unit_price(Money::new(rust_decimal::Decimal::new(2550, 2), Currency::USD))
//!     .dimensions(45.0, 30.0, 15.0)
//!     .weight_kg(2.5)
//!     .build()?;
//!
//! // Serialize to different formats
//! let json = serde_json::to_string_pretty(&product)?;
//! let toml = toml::to_string(&product)?;
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod models;
pub mod algorithms;
pub mod builders;
pub mod pipelines;
pub mod serialization;
pub mod errors;
pub mod enterprise_models;
pub mod analytics;
pub mod realtime;

#[cfg(test)]
pub mod tests;

pub use models::*;
pub use algorithms::*;
pub use builders::*;
pub use pipelines::*;
pub use serialization::*;
pub use errors::*;
pub use enterprise_models::*;
pub use analytics::*;
pub use realtime::*;

/// Convenience module for common imports
pub mod prelude {
    pub use crate::models::*;
    pub use crate::algorithms::*;
    pub use crate::builders::*;
    pub use crate::pipelines::*;
    pub use crate::serialization::*;
    pub use crate::errors::*;
    pub use crate::enterprise_models::*;
    pub use crate::analytics::*;
    pub use crate::realtime::*;
}