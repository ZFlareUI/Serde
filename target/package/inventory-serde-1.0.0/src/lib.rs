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
//! use inventory_serde::prelude::*;
//! use chrono::Utc;
//! use uuid::Uuid;
//!
//! // Create a product using the builder pattern
//! let product = ProductBuilder::new("SKU-001", "Industrial Widget")
//!     .description("High-quality widget for industrial applications")
//!     .category("Manufacturing")
//!     .unit_cost(rust_decimal::Decimal::new(2550, 2), Currency::USD)
//!     .dimensions_cm(45.0, 30.0, 15.0)
//!     .weight_grams(2500)
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

// Re-export all public items from modules
// Note: Ambiguous glob re-exports are expected in comprehensive libraries
#[allow(ambiguous_glob_reexports)]
pub use models::*;
#[allow(ambiguous_glob_reexports)]
pub use algorithms::*;
#[allow(ambiguous_glob_reexports)]
pub use builders::*;
#[allow(ambiguous_glob_reexports)]
pub use pipelines::*;
#[allow(ambiguous_glob_reexports)]
pub use serialization::*;
#[allow(ambiguous_glob_reexports)]
pub use errors::*;
#[allow(ambiguous_glob_reexports)]
pub use enterprise_models::*;
#[allow(ambiguous_glob_reexports)]
pub use analytics::*;
#[allow(ambiguous_glob_reexports)]
pub use realtime::*;

/// Convenience module for common imports
pub mod prelude {
    // Re-export all public items from modules
    // Note: Ambiguous glob re-exports are expected in comprehensive libraries
    #[allow(ambiguous_glob_reexports)]
    pub use crate::models::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::algorithms::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::builders::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::pipelines::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::serialization::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::errors::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::enterprise_models::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::analytics::*;
    #[allow(ambiguous_glob_reexports)]
    pub use crate::realtime::*;
}