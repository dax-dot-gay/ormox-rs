use async_trait::async_trait;

use super::{document::Document, error::OResult};

pub trait OrmoxCollection<T: Document> {}

#[async_trait]
pub trait OrmoxDatabase {
    async fn collection<T: Document>(
        &self,
        name: impl AsRef<str>,
    ) -> OResult<impl OrmoxCollection<T>>;

    async fn collections(&self) -> Vec<String>;
}
