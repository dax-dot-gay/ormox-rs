use async_trait::async_trait;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{document::Document, error::{OResult, OrmoxError}, query::{Query, QueryCompatible}};

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

#[async_trait]
pub trait OrmoxCollection<T: Document> {
    // Identification/inspection functions

    /// What field key is used to store object IDs
    fn id_field(&self) -> String;

    /// The name of this collection
    fn name(&self) -> String;

    // Operation functions
    async fn insert(&self, documents: Vec<&T>) -> OResult<Vec<Uuid>>;
    async fn update(&self, query: impl QueryCompatible, update: impl Serialize + Send, count: OperationCount, upsert: bool) -> OResult<()>;
    async fn delete(&self, query: impl QueryCompatible, count: OperationCount) -> OResult<()>;
    async fn find(&self, query: impl QueryCompatible, options: Find) -> OResult<Vec<T>>;
    async fn all(&self, options: Find) -> OResult<Vec<T>>;

    // Default implementations
    async fn insert_one(&self, document: &T) -> OResult<Uuid> {
        match self.insert(vec![document]).await.and_then(|v| Ok(v.get(0).cloned())) {
            Ok(Some(dc)) => Ok(dc.clone()),
            Ok(None) => Err(OrmoxError::Insert { error: String::from("No documents were inserted.") }),
            Err(e) => Err(e)
        }
    }
    async fn insert_many(&self, documents: Vec<&T>) -> OResult<Vec<Uuid>> {
        self.insert_many(documents).await
    }
    async fn update_one(&self, query: impl QueryCompatible, update: impl Serialize + Send) -> OResult<()> {
        self.update(query, update, OperationCount::One, false).await
    }
    async fn update_many(&self, query: impl QueryCompatible, update: impl Serialize + Send) -> OResult<()> {
        self.update(query, update, OperationCount::Many, false).await
    }
    async fn replace_one(&self, query: impl QueryCompatible, document: impl Document) -> OResult<()> {
        self.update(query, document, OperationCount::One, true).await
    }
    async fn replace_many(&self, query: impl QueryCompatible, document: impl Document) -> OResult<()> {
        self.update(query, document, OperationCount::Many, true).await
    }
    async fn save(&self, document: impl Document) -> OResult<()> {
        self.update(Query::new().equals(self.id_field(), document.id().to_string()).build(), document, OperationCount::One, true).await
    }
    async fn find_one(&self, query: impl QueryCompatible) -> OResult<Option<T>> {
        match self.find(query, Find::one()).await {
            Ok(results) => Ok(results.get(0).cloned()),
            Err(e) => Err(e)
        }
    }
}

#[async_trait]
pub trait OrmoxDatabase {
    async fn collection<T: Document>(
        &self,
        name: impl AsRef<str>,
    ) -> OResult<impl OrmoxCollection<T>>;

    async fn collections(&self) -> Vec<String>;
}
