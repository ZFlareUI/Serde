//! In-memory repository implementation
//!
//! This module provides in-memory implementations of all repository traits
//! for testing, development, and scenarios where persistence is not required.
//! All data is stored in memory using Arc<RwLock<HashMap<>>> for thread safety.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::models::{Product, Warehouse};
use super::{Repository, RepositoryResult, RepositoryError};

/// In-memory storage type
type Storage<T> = Arc<RwLock<HashMap<Uuid, T>>>;

/// In-memory repository factory
pub struct MemoryRepositoryFactory {
    pub products: Storage<Product>,
    pub warehouses: Storage<Warehouse>,
}

impl MemoryRepositoryFactory {
    /// Create new factory with empty storage
    pub fn new() -> Self {
        Self {
            products: Arc::new(RwLock::new(HashMap::new())),
            warehouses: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Clear all data
    pub fn clear_all(&self) {
        self.products.write().unwrap().clear();
        self.warehouses.write().unwrap().clear();
    }

    /// Get product repository
    pub fn product_repository(&self) -> MemoryProductRepository {
        MemoryProductRepository::new(self.products.clone())
    }

    /// Get warehouse repository  
    pub fn warehouse_repository(&self) -> MemoryWarehouseRepository {
        MemoryWarehouseRepository::new(self.warehouses.clone())
    }
}

impl Default for MemoryRepositoryFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Product Repository Implementation
pub struct MemoryProductRepository {
    storage: Storage<Product>,
}

impl MemoryProductRepository {
    pub fn new(storage: Storage<Product>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl Repository<Product> for MemoryProductRepository {
    async fn create(&self, entity: &Product) -> RepositoryResult<Product> {
        let mut storage = self.storage.write()
            .map_err(|_| RepositoryError::internal("Failed to acquire write lock"))?;
            
        if storage.contains_key(&entity.id) {
            return Err(RepositoryError::duplicate("Product", entity.id.to_string()));
        }
        
        // Check for duplicate SKU
        for existing in storage.values() {
            if existing.sku == entity.sku {
                return Err(RepositoryError::duplicate("Product", format!("SKU: {}", entity.sku)));
            }
        }
        
        storage.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn get_by_id(&self, id: Uuid) -> RepositoryResult<Option<Product>> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        Ok(storage.get(&id).cloned())
    }

    async fn update(&self, entity: &Product) -> RepositoryResult<Product> {
        let mut storage = self.storage.write()
            .map_err(|_| RepositoryError::internal("Failed to acquire write lock"))?;
            
        if !storage.contains_key(&entity.id) {
            return Err(RepositoryError::not_found("Product", entity.id.to_string()));
        }
        
        storage.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn delete(&self, id: Uuid) -> RepositoryResult<bool> {
        let mut storage = self.storage.write()
            .map_err(|_| RepositoryError::internal("Failed to acquire write lock"))?;
        Ok(storage.remove(&id).is_some())
    }

    async fn exists(&self, id: Uuid) -> RepositoryResult<bool> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        Ok(storage.contains_key(&id))
    }

    async fn get_all(&self, offset: Option<u64>, limit: Option<u64>) -> RepositoryResult<Vec<Product>> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        let mut items: Vec<Product> = storage.values().cloned().collect();
        
        // Sort by SKU for consistent ordering
        items.sort_by(|a, b| a.sku.cmp(&b.sku));
        
        // Apply pagination
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(1000) as usize;
        
        if offset >= items.len() {
            return Ok(Vec::new());
        }
        
        items.drain(0..offset);
        items.truncate(limit);
        Ok(items)
    }

    async fn count(&self) -> RepositoryResult<u64> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        Ok(storage.len() as u64)
    }
}

/// Warehouse Repository Implementation  
pub struct MemoryWarehouseRepository {
    storage: Storage<Warehouse>,
}

impl MemoryWarehouseRepository {
    pub fn new(storage: Storage<Warehouse>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl Repository<Warehouse> for MemoryWarehouseRepository {
    async fn create(&self, entity: &Warehouse) -> RepositoryResult<Warehouse> {
        let mut storage = self.storage.write()
            .map_err(|_| RepositoryError::internal("Failed to acquire write lock"))?;
            
        if storage.contains_key(&entity.id) {
            return Err(RepositoryError::duplicate("Warehouse", entity.id.to_string()));
        }
        
        // Check for duplicate code
        for existing in storage.values() {
            if existing.code == entity.code {
                return Err(RepositoryError::duplicate("Warehouse", format!("Code: {}", entity.code)));
            }
        }
        
        storage.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn get_by_id(&self, id: Uuid) -> RepositoryResult<Option<Warehouse>> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        Ok(storage.get(&id).cloned())
    }

    async fn update(&self, entity: &Warehouse) -> RepositoryResult<Warehouse> {
        let mut storage = self.storage.write()
            .map_err(|_| RepositoryError::internal("Failed to acquire write lock"))?;
            
        if !storage.contains_key(&entity.id) {
            return Err(RepositoryError::not_found("Warehouse", entity.id.to_string()));
        }
        
        storage.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn delete(&self, id: Uuid) -> RepositoryResult<bool> {
        let mut storage = self.storage.write()
            .map_err(|_| RepositoryError::internal("Failed to acquire write lock"))?;
        Ok(storage.remove(&id).is_some())
    }

    async fn exists(&self, id: Uuid) -> RepositoryResult<bool> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        Ok(storage.contains_key(&id))
    }

    async fn get_all(&self, offset: Option<u64>, limit: Option<u64>) -> RepositoryResult<Vec<Warehouse>> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        let mut items: Vec<Warehouse> = storage.values().cloned().collect();
        
        // Sort by code for consistent ordering
        items.sort_by(|a, b| a.code.cmp(&b.code));
        
        // Apply pagination
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(1000) as usize;
        
        if offset >= items.len() {
            return Ok(Vec::new());
        }
        
        items.drain(0..offset);
        items.truncate(limit);
        Ok(items)
    }

    async fn count(&self) -> RepositoryResult<u64> {
        let storage = self.storage.read()
            .map_err(|_| RepositoryError::internal("Failed to acquire read lock"))?;
        Ok(storage.len() as u64)
    }
}

// Tests are commented out until builder methods are properly integrated
// #[cfg(test)]
// mod tests {
//     use super::*;
//     // Tests will be added when the models are properly integrated
// }