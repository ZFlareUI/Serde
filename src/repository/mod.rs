//! Repository layer for data access
//!
//! This module provides async repository patterns for all domain entities
//! with support for multiple backends (SQL, MongoDB, in-memory) and
//! standardized error handling.

pub mod traits;
pub mod sql;
pub mod mongodb;
pub mod memory;
pub mod error;

// Re-export main types
pub use traits::*;
pub use error::{RepositoryError, RepositoryResult};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

// Model imports are not needed here as they are used in trait definitions

/// Generic repository for any entity type
#[async_trait]
pub trait Repository<T>: Send + Sync {
    /// Create a new entity
    async fn create(&self, entity: &T) -> RepositoryResult<T>;
    
    /// Get entity by ID
    async fn get_by_id(&self, id: Uuid) -> RepositoryResult<Option<T>>;
    
    /// Update an existing entity
    async fn update(&self, entity: &T) -> RepositoryResult<T>;
    
    /// Delete entity by ID
    async fn delete(&self, id: Uuid) -> RepositoryResult<bool>;
    
    /// Check if entity exists
    async fn exists(&self, id: Uuid) -> RepositoryResult<bool>;
    
    /// Get all entities with pagination
    async fn get_all(&self, offset: Option<u64>, limit: Option<u64>) -> RepositoryResult<Vec<T>>;
    
    /// Count total entities
    async fn count(&self) -> RepositoryResult<u64>;
}

/// Search and filter capabilities for repositories
#[async_trait]
pub trait SearchableRepository<T, F>: Repository<T> + Send + Sync {
    /// Search entities with filters
    async fn search(&self, filters: F) -> RepositoryResult<Vec<T>>;
    
    /// Search with pagination
    async fn search_paginated(&self, filters: F, offset: Option<u64>, limit: Option<u64>) -> RepositoryResult<SearchResult<T>>;
    
    /// Count entities matching filters
    async fn count_filtered(&self, filters: F) -> RepositoryResult<u64>;
}

/// Search result with pagination info
#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub offset: u64,
    pub limit: u64,
    pub has_more: bool,
}

impl<T> SearchResult<T> {
    pub fn new(items: Vec<T>, total_count: u64, offset: u64, limit: u64) -> Self {
        let has_more = offset + limit < total_count;
        Self {
            items,
            total_count,
            offset,
            limit,
            has_more,
        }
    }
}