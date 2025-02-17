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

    #[error("Compatibility error.")]
    Compatibility,

    #[error("Not found with query: {query:?}")]
    NotFound {query: String},

    #[error("Failed to parse ID: {provided}")]
    Id {provided: String},

    #[error("Document is uninitialized")]
    Uninitialized
}

pub type OResult<T> = Result<T, OrmoxError>;
