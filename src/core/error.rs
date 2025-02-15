use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrmoxError {
    #[error("Failed to retrieve collection {name:?}: {reason:?}")]
    CollectionRetrieval { name: String, reason: String },

    #[error("Failed to serialize value: {error:?}")]
    Serialization { error: String },
}

pub type OResult<T> = Result<T, OrmoxError>;
