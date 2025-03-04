use futures::stream::TryStreamExt;
use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use mongodb::{
    bson::{self, doc},
    options::IndexOptions,
    Collection, Database, IndexModel,
};
use ormox_core::{
    core::driver::OperationCount, DatabaseDriver, Find, OResult, OrmoxError, Query, Sorting,
};
use uuid::Uuid;

#[allow(dead_code)]
fn wrap<T, E: Error>(result: Result<T, E>) -> OResult<T> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(OrmoxError::driver("base::mongodb", e)),
    }
}

#[allow(dead_code)]
pub struct MongoDriver(Arc<Database>);

#[allow(dead_code)]
impl MongoDriver {
    fn collection(&self, name: String) -> Collection<bson::Document> {
        self.0.collection(name.as_str())
    }

    pub fn new(db: Database) -> Self {
        Self(Arc::new(db))
    }
}

#[async_trait]
impl DatabaseDriver for MongoDriver {
    fn driver_name(&self) -> String {
        String::from("base::mongodb")
    }

    async fn collections(&self) -> OResult<Vec<String>> {
        wrap(self.0.list_collection_names().await)
    }

    async fn insert(
        &self,
        collection: String,
        documents: Vec<bson::Document>,
    ) -> OResult<Vec<Uuid>> {
        let result = wrap(self.collection(collection).insert_many(documents).await)?;
        let mut ids: Vec<Uuid> = Vec::new();
        for id in result.inserted_ids.values() {
            ids.push(wrap(bson::from_bson::<Uuid>(id.clone()))?);
        }
        Ok(ids)
    }

    async fn update(
        &self,
        collection: String,
        query: Query,
        update: bson::Document,
        count: OperationCount,
    ) -> OResult<()> {
        wrap(match count {
            OperationCount::One => {
                self.collection(collection)
                    .update_one(wrap(query.try_into())?, update)
                    .await
            }
            OperationCount::Many => {
                self.collection(collection)
                    .update_many(wrap(query.try_into())?, update)
                    .await
            }
        })?;
        Ok(())
    }

    async fn delete(&self, collection: String, query: Query, count: OperationCount) -> OResult<()> {
        wrap(match count {
            OperationCount::One => {
                self.collection(collection)
                    .delete_one(wrap(query.try_into())?)
                    .await
            }
            OperationCount::Many => {
                self.collection(collection)
                    .delete_many(wrap(query.try_into())?)
                    .await
            }
        })?;
        Ok(())
    }

    async fn find(
        &self,
        collection: String,
        query: Query,
        options: Find,
    ) -> OResult<Vec<bson::Document>> {
        let cl = self.collection(collection);
        let results = match options.operation {
            OperationCount::One => wrap(cl.find_one(wrap(query.try_into())?).await)?
                .and_then(|d| Some(vec![d]))
                .or(Some(Vec::<bson::Document>::new()))
                .unwrap(),
            OperationCount::Many => {
                let mut find = cl.find(wrap(query.try_into())?);
                if let Some(sort) = options.sort {
                    find = find.sort(match sort {
                        Sorting::Ascending(field) => doc! {field: 1},
                        Sorting::Descending(field) => doc! {field: -1},
                    });
                }

                if let Some(skip) = options.offset {
                    find = find.skip(skip.try_into().unwrap());
                }

                if let Some(limit) = options.limit {
                    find = find.limit(limit.try_into().unwrap());
                }

                wrap(wrap(find.await)?.try_collect::<Vec<bson::Document>>().await)?
            }
        };

        Ok(results)
    }

    async fn all(&self, collection: String, options: Find) -> OResult<Vec<bson::Document>> {
        let cl = self.collection(collection);
        let mut find = cl.find(doc! {});
        if let Some(sort) = options.sort {
            find = find.sort(match sort {
                Sorting::Ascending(field) => doc! {field: 1},
                Sorting::Descending(field) => doc! {field: -1},
            });
        }

        if let Some(skip) = options.offset {
            find = find.skip(skip.try_into().unwrap());
        }

        if let Some(limit) = options.limit {
            find = find.limit(limit.try_into().unwrap());
        }

        wrap(wrap(find.await)?.try_collect::<Vec<bson::Document>>().await)
    }

    async fn create_index(&self, collection: String, index: ormox_core::Index) -> OResult<()> {
        let mut keys: bson::Document = bson::Document::new();
        for key in index.fields {
            keys.insert(key, 1);
        }
        wrap(
            self.collection(collection)
                .create_index(
                    IndexModel::builder()
                        .keys(keys)
                        .options(Some(
                            IndexOptions::builder()
                                .unique(Some(index.unique))
                                .name(index.name)
                                .build(),
                        ))
                        .build(),
                )
                .await,
        )
        .and(Ok(()))
    }

    async fn drop_index(&self, collection: String, name: String) -> OResult<()> {
        wrap(self.collection(collection).drop_index(name).await)
    }

    async fn upsert(
        &self,
        collection: String,
        query: Query,
        document: bson::Document,
        count: OperationCount,
    ) -> OResult<()> {
        wrap(match count {
            OperationCount::One => {
                self.collection(collection)
                    .update_one(wrap(query.try_into())?, doc! {"$set": document})
                    .upsert(true)
                    .await
            }
            OperationCount::Many => {
                self.collection(collection)
                    .update_many(wrap(query.try_into())?, doc! {"$set": document})
                    .upsert(true)
                    .await
            }
        })?;
        Ok(())
    }
}
