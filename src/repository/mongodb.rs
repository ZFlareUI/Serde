//! MongoDB-based repository implementations
//!
//! This module provides MongoDB implementations using the mongodb crate.

#[cfg(feature = "mongodb")]
mod implementations {
    use async_trait::async_trait;
    use mongodb::{Client, Database, Collection};
    use uuid::Uuid;
    
    use crate::models::*;
    use super::super::{Repository, RepositoryResult, RepositoryError};

    /// MongoDB repository implementation
    pub struct MongoRepository<T> {
        collection: Collection<T>,
    }

    impl<T> MongoRepository<T> {
        pub fn new(database: &Database, collection_name: impl AsRef<str>) -> Self {
            Self {
                collection: database.collection(collection_name.as_ref()),
            }
        }
    }

    /// MongoDB repository factory
    pub struct MongoRepositoryFactory {
        database: Database,
    }

    impl MongoRepositoryFactory {
        pub fn new(client: Client, database_name: impl AsRef<str>) -> Self {
            Self {
                database: client.database(database_name.as_ref()),
            }
        }
    }
}

#[cfg(not(feature = "mongodb"))]
pub mod placeholder {
    //! Placeholder module when MongoDB feature is not enabled
    
    /// Placeholder MongoDB repository factory
    pub struct MongoRepositoryFactory;
    
    impl MongoRepositoryFactory {
        pub fn new() -> Self {
            Self
        }
    }
}

#[cfg(feature = "mongodb")]
pub use implementations::*;

#[cfg(not(feature = "mongodb"))]
pub use placeholder::*;