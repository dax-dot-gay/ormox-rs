use async_trait::async_trait;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{document::Index, error::{OResult, OrmoxError}, query::Query};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OperationCount {
    One,
    Many
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Sorting {
    Ascending(String),
    Descending(String)
}

impl Sorting {
    pub fn asc(key: impl AsRef<str>) -> Self {
        Self::Ascending(key.as_ref().to_string())
    }

    pub fn desc(key: impl AsRef<str>) -> Self {
        Self::Descending(key.as_ref().to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct Find {
    #[builder(default = "OperationCount::Many")]
    pub operation: OperationCount,

    #[builder(default, setter(into, strip_option))]
    pub offset: Option<usize>,

    #[builder(default, setter(into, strip_option))]
    pub limit: Option<usize>,

    #[builder(default, setter(into, strip_option))]
    pub sort: Option<Sorting>
}

impl Find {
    pub fn many() -> Self {
        Self {
            operation: OperationCount::Many,
            offset: None,
            limit: None,
            sort: None
        }
    }

    pub fn one() -> Self {
        Self {
            operation: OperationCount::One,
            offset: None,
            limit: None,
            sort: None
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
pub trait DatabaseDriver {
    // Metadata functions
    /// Name of this driver (ie "mongodb")
    fn driver_name(&self) -> String;

    // Operation functions
    /// Function to return all collection names
    async fn collections(&self) -> OResult<Vec<String>>;

    /// Base function to insert document(s)
    async fn insert(&self, collection: String, documents: Vec<bson::Document>) -> OResult<Vec<Uuid>>;

    /// Base function to update document(s)
    async fn update(&self, collection: String, query: Query, update: bson::Document, count: OperationCount) -> OResult<()>;

    /// Base function to delete document(s)
    async fn delete(&self, collection: String, query: Query, count: OperationCount) -> OResult<()>;

    /// Base function to find document(s)
    async fn find(&self, collection: String, query: Query, options: Find) -> OResult<Vec<bson::Document>>;

    /// Base function to return all documents in a collection
    async fn all(&self, collection: String, options: Find) -> OResult<Vec<bson::Document>>;

    /// Base function to upsert document(s)
    async fn upsert(&self, collection: String, query: Query, document: bson::Document, count: OperationCount) -> OResult<()>;

    /// Base function to create an index
    async fn create_index(&self, collection: String, index: Index) -> OResult<()> {
        Err(OrmoxError::Unimplemented)
    }

    /// Base function to drop an index
    async fn drop_index(&self, collection: String, name: String) -> OResult<()> {
        Err(OrmoxError::Unimplemented)
    }
}