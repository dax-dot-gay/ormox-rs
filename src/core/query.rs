use std::collections::HashMap;

use bson::{bson, Bson};
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Number, Value};

use super::error::{OResult, OrmoxError};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum QueryOperator {
    Equals { key: String, value: Value },
    NotEquals { key: String, value: Value },
    GreaterThan { key: String, value: Number },
    LessThan { key: String, value: Number },
    GreaterThanEqual { key: String, value: Number },
    LessThanEqual { key: String, value: Number },
    In { key: String, values: Vec<Value> },
    NotIn { key: String, values: Vec<Value> },
    Or { queries: Vec<Query> }
}

impl QueryOperator {
    pub fn key(&self) -> String {
        match self {
            QueryOperator::Equals { key, .. } => key.clone(),
            QueryOperator::NotEquals { key, .. } => key.clone(),
            QueryOperator::GreaterThan { key, .. } => key.clone(),
            QueryOperator::LessThan { key, .. } => key.clone(),
            QueryOperator::GreaterThanEqual { key, .. } => key.clone(),
            QueryOperator::LessThanEqual { key, .. } => key.clone(),
            QueryOperator::In { key, .. } => key.clone(),
            QueryOperator::NotIn { key, .. } => key.clone(),
            QueryOperator::Or { .. } => String::from("$or")
        }
    }
}

impl TryInto<Bson> for QueryOperator {
    type Error = bson::ser::Error;
    fn try_into(self) -> Result<Bson, Self::Error> {
        match self {
            QueryOperator::Equals { value, .. } => bson::to_bson(&value),
            QueryOperator::NotEquals { value, .. } => {
                bson::to_bson(&value).and_then(|v| Ok(bson!({"$ne": v})))
            }
            QueryOperator::GreaterThan { value, .. } => {
                bson::to_bson(&value).and_then(|v| Ok(bson!({"$gt": v})))
            }
            QueryOperator::GreaterThanEqual { value, .. } => {
                bson::to_bson(&value).and_then(|v| Ok(bson!({"$gte": v})))
            }
            QueryOperator::LessThan { value, .. } => {
                bson::to_bson(&value).and_then(|v| Ok(bson!({"$lt": v})))
            }
            QueryOperator::LessThanEqual { value, .. } => {
                bson::to_bson(&value).and_then(|v| Ok(bson!({"$lte": v})))
            }
            QueryOperator::In { values, .. } => {
                bson::to_bson(&values).and_then(|v| Ok(bson!({"$in": v})))
            }
            QueryOperator::NotIn { values, .. } => {
                bson::to_bson(&values).and_then(|v| Ok(bson!({"$nin": v})))
            }
            QueryOperator::Or { queries } => {
                let mut result: Vec<Bson> = Vec::new();
                for q in queries {
                    result.push(TryInto::<Bson>::try_into(q)?);
                }

                Ok(Bson::Array(result))
            }
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Query(HashMap<String, QueryOperator>);

impl TryInto<Bson> for Query {
    type Error = bson::ser::Error;
    fn try_into(self) -> Result<Bson, Self::Error> {
        let mut result = bson::Document::new();
        for (key, value) in self.0 {
            result.insert(key, TryInto::<Bson>::try_into(value)?);
        }
        Ok(Bson::Document(result))
    }
}

impl Query {
    fn push(&mut self, operator: QueryOperator) -> &mut Self {
        self.0.insert(operator.key(), operator);
        self
    }

    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn equals<K: AsRef<str>, V: Into<Value>>(&mut self, key: K, value: V) -> &mut Self {
        self.push(QueryOperator::Equals {
            key: key.as_ref().to_string(),
            value: value.into(),
        })
    }

    pub fn not_equals<K: AsRef<str>, V: Into<Value>>(&mut self, key: K, value: V) -> &mut Self {
        self.push(QueryOperator::NotEquals {
            key: key.as_ref().to_string(),
            value: value.into(),
        })
    }

    pub fn greater_than<K: AsRef<str>, V: Into<Number>>(&mut self, key: K, value: V) -> &mut Self {
        self.push(QueryOperator::GreaterThan {
            key: key.as_ref().to_string(),
            value: value.into(),
        })
    }

    pub fn greater_than_equal<K: AsRef<str>, V: Into<Number>>(
        &mut self,
        key: K,
        value: V,
    ) -> &mut Self {
        self.push(QueryOperator::GreaterThanEqual {
            key: key.as_ref().to_string(),
            value: value.into(),
        })
    }

    pub fn less_than<K: AsRef<str>, V: Into<Number>>(&mut self, key: K, value: V) -> &mut Self {
        self.push(QueryOperator::LessThan {
            key: key.as_ref().to_string(),
            value: value.into(),
        })
    }

    pub fn less_than_equal<K: AsRef<str>, V: Into<Number>>(
        &mut self,
        key: K,
        value: V,
    ) -> &mut Self {
        self.push(QueryOperator::LessThanEqual {
            key: key.as_ref().to_string(),
            value: value.into(),
        })
    }

    pub fn any_of<K: AsRef<str>, V: Into<Value>>(&mut self, key: K, value: Vec<V>) -> &mut Self {
        self.push(QueryOperator::In {
            key: key.as_ref().to_string(),
            values: value.into_iter().map(|v| Into::<Value>::into(v)).collect(),
        })
    }

    pub fn none_of<K: AsRef<str>, V: Into<Value>>(&mut self, key: K, value: Vec<V>) -> &mut Self {
        self.push(QueryOperator::NotIn {
            key: key.as_ref().to_string(),
            values: value.into_iter().map(|v| Into::<Value>::into(v)).collect(),
        })
    }

    pub fn or(&mut self, query: Query) -> &mut Self {
        let mut current = if let QueryOperator::Or { queries } = self
            .0
            .get("$or")
            .or(Some(&QueryOperator::Or {
                queries: Vec::new(),
            }))
            .unwrap()
            .clone()
        {
            queries
        } else {
            Vec::<Query>::new()
        };

        current.push(query);
        self.push(QueryOperator::Or {
            queries: current.clone(),
        })
    }

    pub fn build(&mut self) -> Self {
        self.clone()
    }
}

pub trait QueryCompatible : TryInto<Query> + TryFrom<Query> + Send {
    fn into(self) -> OResult<Query> {
        match TryInto::<Query>::try_into(self) {
            Ok(r) => Ok(r),
            Err(_) => Err(OrmoxError::Compatibility )
        }
    }

    fn from(query: Query) -> OResult<Self> {
        match Self::try_from(query) {
            Ok(r) => Ok(r),
            Err(_) => Err(OrmoxError::Compatibility )
        }
    }
}

impl QueryCompatible for Query {}

impl TryInto<bson::Document> for Query {
    type Error = bson::ser::Error;
    fn try_into(self) -> Result<bson::Document, Self::Error> {
        let bs = TryInto::<Bson>::try_into(self)?;
        Ok(bs.as_document().expect("TryInto<Bson> should return a Document type, but did not.").clone())
    }
}

impl TryFrom<bson::Document> for Query {
    type Error = OrmoxError;
    fn try_from(value: bson::Document) -> Result<Self, Self::Error> {
        let mut query = Query::new();
        for (key, item) in value {
            if key == "$or" {
                let doc_array = item.as_array().ok_or(OrmoxError::Deserialization { error: String::from("Value of $or key was not an array.") })?;
                for case in doc_array {
                    let case_doc = case.as_document().ok_or(OrmoxError::Deserialization { error: String::from("A case within an $or clause was not a document.") })?.clone();
                    query.or(Query::try_from(case_doc)?);
                }
            } else {
                if let Some(subdoc) = item.as_document() {

                } else {
                    query.equals(key, to_value(item.clone()).or_else(|e| Err(OrmoxError::Deserialization { error: e.to_string() }))?);
                }
            }
        }

        Ok(query.build())
    }
}
