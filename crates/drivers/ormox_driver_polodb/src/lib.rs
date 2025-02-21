use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use ormox_core::bson::doc;
use ormox_core::{bson, Find, Sorting};
use ormox_core::core::driver::OperationCount;
use ormox_core::{DatabaseDriver, OResult, OrmoxError, Query};
use polodb_core::options::UpdateOptions;
use polodb_core::{Collection, CollectionT, Database};
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
        count: OperationCount,
        upsert: bool,
    ) -> OResult<()> {
        wrap(match count {
            OperationCount::One => self.collection(collection).update_one_with_options(
                wrap(query.try_into())?,
                update,
                UpdateOptions::builder().upsert(upsert).build(),
            ),
            OperationCount::Many => self.collection(collection).update_many_with_options(
                wrap(query.try_into())?,
                update,
                UpdateOptions::builder().upsert(upsert).build(),
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

    async fn find(&self, collection: String, query: Query, options: Find) -> OResult<Vec<bson::Document>> {
        let cl = self.collection(collection);
        let results = match options.operation {
            OperationCount::One => wrap(cl.find_one(wrap(query.try_into())?))?.and_then(|d| Some(vec![d])).or(Some(Vec::<bson::Document>::new())).unwrap(),
            OperationCount::Many => {
                let mut find = cl.find(wrap(query.try_into())?);
                if let Some(sort) = options.sort {
                    find = find.sort(match sort {
                        Sorting::Ascending(field) => doc! {field: 1},
                        Sorting::Descending(field) => doc! {field: -1}
                    });
                }

                if let Some(skip) = options.offset {
                    find = find.skip(skip.try_into().unwrap());
                }

                if let Some(limit) = options.limit {
                    find = find.limit(limit.try_into().unwrap());
                }

                wrap(find.run())?.filter(|r| r.is_ok()).map(|r| r.unwrap()).collect()
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
                Sorting::Descending(field) => doc! {field: -1}
            });
        }

        if let Some(skip) = options.offset {
            find = find.skip(skip.try_into().unwrap());
        }

        if let Some(limit) = options.limit {
            find = find.limit(limit.try_into().unwrap());
        }

        Ok(wrap(find.run())?.filter(|r| r.is_ok()).map(|r| r.unwrap()).collect())
    }
}
