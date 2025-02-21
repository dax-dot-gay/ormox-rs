use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use ormox_core::bson::doc;
use ormox_core::core::driver::OperationCount;
use ormox_core::{bson, Find, Sorting};
use ormox_core::{DatabaseDriver, OResult, OrmoxError, Query};
use polodb_core::options::UpdateOptions;
use polodb_core::{Collection, CollectionT, Database, IndexModel, IndexOptions};
use uuid::Uuid;

#[allow(dead_code)]
fn wrap<T, E: Error>(result: Result<T, E>) -> OResult<T> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(OrmoxError::driver("base::polodb", e)),
    }
}

#[allow(dead_code)]
pub struct PoloDriver(Arc<Database>);

#[allow(dead_code)]
impl PoloDriver {
    fn collection(&self, name: String) -> Collection<bson::Document> {
        self.0.collection(&name)
    }

    pub fn new(database_path: impl AsRef<str>) -> OResult<Self> {
        let db = wrap(Database::open_path(database_path.as_ref().to_string()))?;
        Ok(Self(Arc::new(db)))
    }
}

#[async_trait]
impl DatabaseDriver for PoloDriver {
    fn driver_name(&self) -> String {
        String::from("base::polodb")
    }

    async fn collections(&self) -> OResult<Vec<String>> {
        wrap(self.0.list_collection_names())
    }

    async fn insert(
        &self,
        collection: String,
        documents: Vec<bson::Document>,
    ) -> OResult<Vec<Uuid>> {
        let result = wrap(self.collection(collection).insert_many(documents))?;
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
        count: OperationCount
    ) -> OResult<()> {
        wrap(match count {
            OperationCount::One => self.collection(collection).update_one(
                wrap(query.try_into())?,
                update
            ),
            OperationCount::Many => self.collection(collection).update_many(
                wrap(query.try_into())?,
                update
            ),
        })?;
        Ok(())
    }

    async fn delete(&self, collection: String, query: Query, count: OperationCount) -> OResult<()> {
        wrap(match count {
            OperationCount::One => self
                .collection(collection)
                .delete_one(wrap(query.try_into())?),
            OperationCount::Many => self
                .collection(collection)
                .delete_many(wrap(query.try_into())?),
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
            OperationCount::One => wrap(cl.find_one(wrap(query.try_into())?))?
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

                wrap(find.run())?
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap())
                    .collect()
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

        Ok(wrap(find.run())?
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect())
    }

    async fn create_index(&self, collection: String, index: ormox_core::Index) -> OResult<()> {
        let mut keys: bson::Document = bson::Document::new();
        for key in index.fields {
            keys.insert(key, 1);
        }
        wrap(self.collection(collection).create_index(IndexModel {
            keys,
            options: Some(IndexOptions {
                name: index.name,
                unique: if index.unique { Some(true) } else { None },
            }),
        }))
    }

    async fn drop_index(&self, collection: String, name: String) -> OResult<()> {
        wrap(self.collection(collection).drop_index(name))
    }

    async fn upsert(
        &self,
        collection: String,
        query: Query,
        document: bson::Document,
        count: OperationCount
    ) -> OResult<()> {
        wrap(match count {
            OperationCount::One => self.collection(collection).update_one_with_options(
                wrap(query.try_into())?,
                doc! {"$set": document},
                UpdateOptions::builder().upsert(true).build()
            ),
            OperationCount::Many => self.collection(collection).update_many_with_options(
                wrap(query.try_into())?,
                doc! {"$set": document},
                UpdateOptions::builder().upsert(true).build()
            ),
        })?;
        Ok(())
    }
}
