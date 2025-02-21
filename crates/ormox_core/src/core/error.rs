use std::fmt::{Debug, Display};

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum OrmoxError {
    #[error("Failed to retrieve collection {name:?}: {reason:?}")]
    CollectionRetrieval { name: String, reason: String },

    #[error("Failed to serialize value: {error:?}")]
    Serialization { error: String },

    #[error("Failed to deserialize value: {error:?}")]
    Deserialization { error: String },

    #[error("Failed to insert document: {error:?}")]
    Insert {error: String},

    #[error("Compatibility error: {error:?}")]
    Compatibility {error: String},

    #[error("Not found with query: {query:?}")]
    NotFound {query: String},

    #[error("Failed to parse ID: {provided}")]
    Id {provided: String},

    #[error("Document is uninitialized")]
    Uninitialized,

    #[error("Method is not implemented on this driver")]
    Unimplemented,

    #[error("Driver-specific error: {driver_name}: {error:?}")]
    Driver {driver_name: String, error: String}
}

impl OrmoxError {
    pub fn serialization(error: impl Display) -> Self {
        Self::Serialization { error: error.to_string() }
    }

    pub fn deserialization(error: impl Display) -> Self {
        Self::Deserialization { error: error.to_string() }
    }

    pub fn insert(error: impl Display) -> Self {
        Self::Insert { error: error.to_string() }
    }

    pub fn compaibility(error: impl Display) -> Self {
        Self::Compatibility { error: error.to_string() }
    }

    pub fn not_found(query: impl AsRef<str>) -> Self {
        Self::NotFound { query: query.as_ref().to_string() }
    }

    pub fn id(id: impl AsRef<str>) -> Self {
        Self::Id { provided: id.as_ref().to_string() }
    }

    pub fn driver(driver: impl AsRef<str>, error: impl std::error::Error) -> Self {
        Self::Driver { driver_name: driver.as_ref().to_string(), error: error.to_string() }
    }
}

pub type OResult<T> = Result<T, OrmoxError>;
