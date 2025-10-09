//! Repository error types
//!
//! This module defines error types used throughout the repository layer
//! with support for different database backends and error mapping.

// Error module - no unused imports
use thiserror::Error;

/// Repository operation result type
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Repository-specific errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    /// Entity not found
    #[error("Entity not found: {entity_type} with ID {id}")]
    NotFound { entity_type: String, id: String },

    /// Duplicate entity (unique constraint violation)
    #[error("Duplicate entity: {entity_type} with key {key} already exists")]
    Duplicate { entity_type: String, key: String },

    /// Invalid operation
    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    /// Validation error
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    /// Database connection error
    #[error("Database connection error: {message}")]
    Connection { message: String },

    /// Database query error
    #[error("Database query error: {message}")]
    Query { message: String },

    /// Transaction error
    #[error("Transaction error: {message}")]
    Transaction { message: String },

    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Optimistic locking conflict
    #[error("Optimistic lock conflict: {entity_type} with ID {id} was modified by another process")]
    OptimisticLock { entity_type: String, id: String },

    /// Timeout error
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Migration error
    #[error("Migration error: {message}")]
    Migration { message: String },

    /// Generic internal error
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl RepositoryError {
    /// Create a not found error
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create a duplicate error
    pub fn duplicate(entity_type: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Duplicate {
            entity_type: entity_type.into(),
            key: key.into(),
        }
    }

    /// Create an invalid operation error
    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::InvalidOperation {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a query error
    pub fn query(message: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
        }
    }

    /// Create a transaction error
    pub fn transaction(message: impl Into<String>) -> Self {
        Self::Transaction {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Create an optimistic lock error
    pub fn optimistic_lock(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::OptimisticLock {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a migration error
    pub fn migration(message: impl Into<String>) -> Self {
        Self::Migration {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Connection { .. } => true,
            Self::Timeout { .. } => true,
            Self::Transaction { .. } => true,
            _ => false,
        }
    }

    /// Check if error indicates a temporary issue
    pub fn is_temporary(&self) -> bool {
        match self {
            Self::Connection { .. } => true,
            Self::Timeout { .. } => true,
            Self::OptimisticLock { .. } => true,
            _ => false,
        }
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "not_found",
            Self::Duplicate { .. } => "duplicate",
            Self::InvalidOperation { .. } => "invalid_operation",
            Self::Validation { .. } => "validation",
            Self::Connection { .. } => "connection",
            Self::Query { .. } => "query",
            Self::Transaction { .. } => "transaction",
            Self::Serialization { .. } => "serialization",
            Self::OptimisticLock { .. } => "optimistic_lock",
            Self::Timeout { .. } => "timeout",
            Self::Configuration { .. } => "configuration",
            Self::Migration { .. } => "migration",
            Self::Internal { .. } => "internal",
        }
    }
}

// Error conversions for different database backends

#[cfg(feature = "sql")]
impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        use sqlx::Error;
        
        match error {
            Error::RowNotFound => Self::not_found("Entity", "unknown"),
            Error::Database(db_error) => {
                let error_code = db_error.code();
                let message = db_error.message();
                
                // Handle common constraint violations
                if let Some(code) = error_code {
                    match code.as_ref() {
                        // PostgreSQL unique violation
                        "23505" => Self::duplicate("Entity", "unknown"),
                        // PostgreSQL foreign key violation
                        "23503" => Self::validation("foreign_key", message),
                        // PostgreSQL check violation
                        "23514" => Self::validation("check_constraint", message),
                        // MySQL duplicate entry
                        "1062" => Self::duplicate("Entity", "unknown"),
                        // SQLite constraint violation
                        "1555" => Self::validation("constraint", message),
                        _ => Self::query(format!("Database error {}: {}", code, message)),
                    }
                } else {
                    Self::query(message)
                }
            },
            Error::Io(io_error) => Self::connection(io_error.to_string()),
            Error::Tls(tls_error) => Self::connection(tls_error.to_string()),
            Error::Protocol(msg) => Self::connection(msg),
            Error::Configuration(config_error) => Self::configuration(config_error.to_string()),
            Error::PoolTimedOut => Self::timeout("Connection pool"),
            Error::PoolClosed => Self::connection("Connection pool is closed"),
            Error::WorkerCrashed => Self::internal("Database worker crashed"),
            _ => Self::internal(error.to_string()),
        }
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::error::Error> for RepositoryError {
    fn from(error: mongodb::error::Error) -> Self {
        use mongodb::error::{Error, ErrorKind};
        
        match error.kind.as_ref() {
            ErrorKind::Authentication { .. } => Self::connection("Authentication failed"),
            ErrorKind::BulkWrite(bulk_error) => {
                if bulk_error.write_errors.iter().any(|e| e.code == 11000) {
                    Self::duplicate("Entity", "unknown")
                } else {
                    Self::query(bulk_error.to_string())
                }
            },
            ErrorKind::Command(command_error) => {
                match command_error.code {
                    11000 => Self::duplicate("Entity", "unknown"), // Duplicate key
                    _ => Self::query(command_error.message.clone()),
                }
            },
            ErrorKind::InvalidArgument { .. } => Self::validation("argument", error.to_string()),
            ErrorKind::Io(io_error) => Self::connection(io_error.to_string()),
            ErrorKind::ServerSelection { .. } => Self::connection("Server selection failed"),
            ErrorKind::Transaction { .. } => Self::transaction(error.to_string()),
            _ => Self::internal(error.to_string()),
        }
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(error: serde_json::Error) -> Self {
        Self::serialization(error.to_string())
    }
}

impl From<uuid::Error> for RepositoryError {
    fn from(error: uuid::Error) -> Self {
        Self::validation("uuid", error.to_string())
    }
}

impl From<chrono::ParseError> for RepositoryError {
    fn from(error: chrono::ParseError) -> Self {
        Self::validation("datetime", error.to_string())
    }
}

impl From<rust_decimal::Error> for RepositoryError {
    fn from(error: rust_decimal::Error) -> Self {
        Self::validation("decimal", error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let not_found = RepositoryError::not_found("Product", "123");
        assert!(matches!(not_found, RepositoryError::NotFound { .. }));
        assert_eq!(not_found.category(), "not_found");
    }

    #[test]
    fn test_error_retryable() {
        let connection_error = RepositoryError::connection("Connection lost");
        assert!(connection_error.is_retryable());
        assert!(connection_error.is_temporary());

        let validation_error = RepositoryError::validation("name", "Required field");
        assert!(!validation_error.is_retryable());
        assert!(!validation_error.is_temporary());
    }

    #[test]
    fn test_error_display() {
        let error = RepositoryError::duplicate("Product", "SKU-001");
        let display = format!("{}", error);
        assert!(display.contains("Duplicate entity"));
        assert!(display.contains("Product"));
        assert!(display.contains("SKU-001"));
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(RepositoryError::not_found("", "").category(), "not_found");
        assert_eq!(RepositoryError::validation("", "").category(), "validation");
        assert_eq!(RepositoryError::connection("").category(), "connection");
        assert_eq!(RepositoryError::timeout("").category(), "timeout");
    }
}