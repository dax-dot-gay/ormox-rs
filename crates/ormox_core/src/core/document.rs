use std::fmt::Debug;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use crate::client::{Client, Collection};

use super::{error::{OResult, OrmoxError}, query::Query};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Index {
    pub fields: Vec<String>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub unique: bool
}

impl Index {
    pub fn new(field: impl AsRef<str>) -> Self {
        Self {
            fields: vec![field.as_ref().to_string()],
            name: None,
            unique: false
        }
    }

    pub fn new_compound(fields: Vec<String>) -> Self {
        let mut f = fields.clone();
        f.sort();
        f.dedup();
        Self {
            fields: f,
            name: None,
            unique: false
        }
    }

    pub fn named(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    pub fn unnamed(&mut self) -> &mut Self {
        self.name = None;
        self
    }

    pub fn unique(&mut self, unique: bool) -> &mut Self {
        self.unique = unique;
        self
    }

    pub fn field(&mut self, field: impl AsRef<str>) -> &mut Self {
        if !self.fields.contains(&field.as_ref().to_string()) {
            self.fields.push(field.as_ref().to_string());
            self.fields.sort();
        }

        self
    }

    pub fn build(&mut self) -> Self {
        self.clone()
    }
}

#[async_trait::async_trait]
pub trait Document: Serialize + DeserializeOwned + Clone + Debug + Sync + Send {
    fn id(&self) -> Uuid;
    fn collection_name() -> String;
    fn indexes() -> Vec<Index>;
    fn parse(data: bson::Document, collection: Option<Collection<Self>>) -> OResult<Self>;
    fn attached_collection(&self) -> Option<Collection<Self>>;
    fn attach_collection(&self, collection: Collection<Self>) -> ();
    fn collection(&self) -> Option<Collection<Self>> {
        if let Some(attached) = self.attached_collection() {
            Some(attached)
        } else if let Some(global) = Client::global() {
            Some(global.collection::<Self>())
        } else {
            None
        }
    }

    async fn save(&self) -> OResult<()> {
        if let Some(collection) = self.collection() {
            collection.save(self.clone()).await
        } else {
            Err(OrmoxError::Uninitialized)
        }
    }

    async fn delete(self) -> OResult<()> {
        if let Some(collection) = self.collection() {
            collection.delete_one(Query::new().field(collection.driver().id_field(), self.id().to_string()).build()).await
        } else {
            Err(OrmoxError::Uninitialized)
        }
    }
}