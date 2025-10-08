use thiserror::Error;

/// Comprehensive error types for the inventory management library
#[derive(Error, Debug, Clone, PartialEq)]
pub enum InventoryError {
    /// Validation errors for business logic constraints
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Product-related errors
    #[error("Product error: {message}")]
    Product { message: String },

    /// Supplier-related errors
    #[error("Supplier error: {message}")]
    Supplier { message: String },

    /// Inventory calculation errors
    #[error("Calculation error: {message}")]
    Calculation { message: String },

    /// Currency conversion errors
    #[error("Currency error: {message}")]
    Currency { message: String },

    /// Data pipeline processing errors
    #[error("Pipeline error: {message}")]
    Pipeline { message: String },

    /// Builder pattern validation errors
    #[error("Builder error: {message}")]
    Builder { message: String },
}

impl InventoryError {
    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Create a new product error
    pub fn product<S: Into<String>>(message: S) -> Self {
        Self::Product {
            message: message.into(),
        }
    }

    /// Create a new supplier error
    pub fn supplier<S: Into<String>>(message: S) -> Self {
        Self::Supplier {
            message: message.into(),
        }
    }

    /// Create a new calculation error
    pub fn calculation<S: Into<String>>(message: S) -> Self {
        Self::Calculation {
            message: message.into(),
        }
    }

    /// Create a new currency error
    pub fn currency<S: Into<String>>(message: S) -> Self {
        Self::Currency {
            message: message.into(),
        }
    }

    /// Create a new pipeline error
    pub fn pipeline<S: Into<String>>(message: S) -> Self {
        Self::Pipeline {
            message: message.into(),
        }
    }

    /// Create a new builder error
    pub fn builder<S: Into<String>>(message: S) -> Self {
        Self::Builder {
            message: message.into(),
        }
    }
}

/// Result type alias for inventory operations
pub type InventoryResult<T> = Result<T, InventoryError>;

// Implement conversions from common error types
impl From<serde_json::Error> for InventoryError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization(format!("JSON error: {}", err))
    }
}

impl From<toml::de::Error> for InventoryError {
    fn from(err: toml::de::Error) -> Self {
        Self::serialization(format!("TOML deserialization error: {}", err))
    }
}

impl From<toml::ser::Error> for InventoryError {
    fn from(err: toml::ser::Error) -> Self {
        Self::serialization(format!("TOML serialization error: {}", err))
    }
}

impl From<csv::Error> for InventoryError {
    fn from(err: csv::Error) -> Self {
        Self::serialization(format!("CSV error: {}", err))
    }
}