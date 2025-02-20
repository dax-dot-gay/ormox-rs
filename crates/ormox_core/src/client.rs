use std::{error::Error, marker::PhantomData, sync::Arc};
use serde::Serialize;

use uuid::Uuid;

use crate::{
    core::{
        document::{Document, Index},
        driver::{DatabaseDriver, Find, OperationCount},
        error::{OResult, OrmoxError},
        query::Query,
    },
    ORMOX,
};

#[derive(Clone)]
pub struct Client(Arc<dyn DatabaseDriver + Send + Sync>);

impl Client {
    pub fn create<D: DatabaseDriver + Send + Sync + 'static>(driver: D) -> Arc<Self> {
        Arc::new(Self(Arc::new(driver)))
    }

    pub fn create_global<D: DatabaseDriver + Send + Sync + 'static>(driver: D) -> Arc<Self> {
        if let Ok(_) = ORMOX.set(Self::create(driver)) {
            ORMOX.get().unwrap().clone()
        } else {
            panic!("Global instance already set!");
        }
    }

    pub fn global() -> Option<Arc<Self>> {
        ORMOX.get().cloned()
    }

    pub fn driver(&self) -> Arc<dyn DatabaseDriver + Send + Sync> {
        self.0.clone()
    }

    pub async fn collections(&self) -> OResult<Vec<String>> {
        self.driver().collections().await
    }

    pub fn collection<D: Document>(&self) -> Collection<D> {
        Collection::<D>::new(self.clone())
    }
}

#[derive(Clone)]
pub struct Collection<T: Document>(Client, PhantomData<T>);

impl<T: Document> Collection<T> {
    pub fn client(&self) -> Client {
        self.0.clone()
    }

    pub fn driver(&self) -> Arc<dyn DatabaseDriver + Send + Sync> {
        self.client().driver()
    }

    pub fn new(client: Client) -> Self {
        Self(client, PhantomData)
    }

    pub fn name(&self) -> String {
        T::collection_name().clone()
    }

    pub async fn find(
        &self,
        query: impl TryInto<Query, Error = impl Error>,
        options: Option<Find>,
    ) -> OResult<Vec<T>> {
        let raw = self
            .driver()
            .find(self.name(), query.try_into().or_else(|e| Err(OrmoxError::Compatibility { error: e.to_string() }))?, options.unwrap_or(Find::many()))
            .await?;

        let mut results: Vec<T> = Vec::new();
        for r in raw {
            results.push(T::parse(r, Some(self.clone()))?);
        }
        Ok(results)
    }

    pub async fn all(&self, options: Option<Find>) -> OResult<Vec<T>> {
        let raw = self
            .driver()
            .all(self.name(), options.unwrap_or(Find::many()))
            .await?;

        let mut results: Vec<T> = Vec::new();
        for r in raw {
            results.push(T::parse(r, Some(self.clone()))?);
        }
        Ok(results)
    }

    pub async fn insert(&self, docs: Vec<T>) -> OResult<Vec<Uuid>> {
        let mut serialized: Vec<bson::Document> = Vec::new();
        for d in docs {
            serialized.push(bson::to_document(&d).or_else(|e| {
                Err(OrmoxError::Serialization {
                    error: e.to_string(),
                })
            })?);
        }

        self.driver().insert(self.name(), serialized).await
    }

    pub async fn update(
        &self,
        query: impl TryInto<Query, Error = impl Error>,
        update: impl Serialize,
        operations: OperationCount,
        upsert: bool,
    ) -> OResult<()> {
        self.driver()
            .update(
                self.name(),
                query.try_into().or_else(|e| Err(OrmoxError::Compatibility { error: e.to_string() }))?,
                bson::to_document(&update).or_else(|e| {
                    Err(OrmoxError::Deserialization {
                        error: e.to_string(),
                    })
                })?,
                operations,
                upsert,
            )
            .await
    }

    pub async fn delete(
        &self,
        query: impl TryInto<Query, Error = impl Error>,
        operations: OperationCount,
    ) -> OResult<()> {
        self.driver()
            .delete(self.name(), query.try_into().or_else(|e| Err(OrmoxError::Compatibility { error: e.to_string() }))?, operations)
            .await
    }

    pub async fn find_one(&self, query: impl TryInto<Query, Error = impl Error>) -> OResult<T> {
        let _query: Query = query.try_into().or_else(|e| Err(OrmoxError::Compatibility { error: e.to_string() }))?;
        if let Some(result) = self.find(_query.clone(), Some(Find::one())).await?.get(0) {
            Ok(result.clone())
        } else {
            Err(OrmoxError::NotFound {
                query: TryInto::<bson::Document>::try_into(_query).and_then(|d| Ok(d.to_string())).or::<()>(Ok(String::from("Unparseable query"))).unwrap(),
            })
        }
    }

    pub async fn find_many(&self, query: impl TryInto<Query, Error = impl Error>) -> OResult<Vec<T>> {
        self.find(query, Some(Find::many())).await
    }

    pub async fn get(&self, id: impl AsRef<str>) -> OResult<T> {
        self.find_one(
            Query::new()
                .field(T::id_field(), id.as_ref().to_string())
                .build(),
        )
        .await
    }

    pub async fn save(&self, document: T) -> OResult<()> {
        self.update(
            Query::new()
                .field(T::id_field(), document.id().to_string())
                .build(),
            document,
            OperationCount::One,
            true,
        )
        .await
    }

    pub async fn delete_one(&self, query: impl TryInto<Query, Error = impl Error>) -> OResult<()> {
        self.delete(query, OperationCount::One).await
    }

    pub async fn delete_many(&self, query: impl TryInto<Query, Error = impl Error>) -> OResult<()> {
        self.delete(query, OperationCount::Many).await
    }

    pub async fn create_index(&self, index: Index) -> OResult<()> {
        self.driver().create_index(self.name(), index).await
    }

    pub async fn drop_index(&self, index_name: impl AsRef<str>) -> OResult<()> {
        self.driver().drop_index(self.name(), index_name.as_ref().to_string()).await
    }
}
