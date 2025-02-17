use std::{marker::PhantomData, sync::Arc};

use bson::Bson;
use serde::Serialize;

use uuid::Uuid;

use crate::{
    core::{
        document::Document,
        driver::{DatabaseDriver, Find, OperationCount},
        error::{OResult, OrmoxError},
        query::{Query, QueryCompatible},
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
        query: impl QueryCompatible,
        options: Option<Find>,
    ) -> OResult<Vec<T>> {
        let raw = self
            .driver()
            .find(self.name(), query.into()?, options.unwrap_or(Find::many()))
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
        query: impl QueryCompatible,
        update: impl Serialize,
        operations: OperationCount,
        upsert: bool,
    ) -> OResult<()> {
        self.driver()
            .update(
                self.name(),
                query.into()?,
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
        query: impl QueryCompatible,
        operations: OperationCount,
    ) -> OResult<()> {
        self.driver()
            .delete(self.name(), query.into()?, operations)
            .await
    }

    pub async fn find_one(&self, query: impl QueryCompatible) -> OResult<T> {
        let _query = query.into()?;
        if let Some(result) = self.find(_query.clone(), Some(Find::one())).await?.get(0) {
            Ok(result.clone())
        } else {
            Err(OrmoxError::NotFound {
                query: TryInto::<Bson>::try_into(_query)
                    .and_then(|r| Ok(r.to_string()))
                    .or::<()>(Ok(String::from("Unparseable query")))
                    .unwrap(),
            })
        }
    }

    pub async fn find_many(&self, query: impl QueryCompatible) -> OResult<Vec<T>> {
        self.find(query, Some(Find::many())).await
    }

    pub async fn get(&self, id: impl AsRef<str>) -> OResult<T> {
        self.find_one(
            Query::new()
                .equals(self.driver().id_field(), id.as_ref().to_string())
                .build(),
        )
        .await
    }

    pub async fn save(&self, document: T) -> OResult<()> {
        self.update(
            Query::new()
                .equals(self.driver().id_field(), document.id().to_string())
                .build(),
            document,
            OperationCount::One,
            true,
        )
        .await
    }

    pub async fn delete_one(&self, query: impl QueryCompatible) -> OResult<()> {
        self.delete(query, OperationCount::One).await
    }

    pub async fn delete_many(&self, query: impl QueryCompatible) -> OResult<()> {
        self.delete(query, OperationCount::Many).await
    }
}
