use std::collections::HashMap;

use bson::Bson;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Number, Value};

use super::error::{OResult, OrmoxError};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum QueryKey {
    String(String),
    Operator(String),
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Equals,
    NotEquals,
    In,
    NotIn,
    And,
    Or,
    Not,
}

impl ToString for QueryKey {
    fn to_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Operator(o) => o.clone(),
            Self::GreaterThan => "$gt".into(),
            Self::LessThan => "$lt".into(),
            Self::GreaterThanEqual => "$gte".into(),
            Self::LessThanEqual => "$lte".into(),
            Self::Equals => "$eq".into(),
            Self::NotEquals => "$ne".into(),
            Self::In => "$in".into(),
            Self::NotIn => "$nin".into(),
            Self::And => "$and".into(),
            Self::Or => "$or".into(),
            Self::Not => "$not".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QueryValue {
    Value(Value),
    Casematch(Vec<Query>),
    Mapping(Query),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Query(HashMap<QueryKey, QueryValue>);

impl From<&Query> for Query {
    fn from(value: &Query) -> Self {
        value.clone()
    }
}

impl Query {
    pub fn new() -> Self {
        Query(HashMap::new())
    }

    fn push(&mut self, key: QueryKey, value: QueryValue) -> &mut Self {
        let _ = self.0.insert(key.clone(), value.clone());
        self
    }

    pub fn field(&mut self, key: impl AsRef<str>, value: impl Into<Value>) -> &mut Self {
        self.push(
            QueryKey::String(key.as_ref().to_string()),
            QueryValue::Value(value.into()),
        )
    }

    pub fn subquery(&mut self, key: impl AsRef<str>, child: impl Into<Query>) -> &mut Self {
        self.push(
            QueryKey::String(key.as_ref().to_string()),
            QueryValue::Mapping(child.into()),
        )
    }

    pub fn operation(&mut self, operation: impl AsRef<str>, value: QueryValue) -> &mut Self {
        self.push(
            QueryKey::Operator(operation.as_ref().to_string()),
            value.clone(),
        )
    }

    pub fn greater_than(&mut self, value: impl Into<Number>) -> &mut Self {
        self.push(
            QueryKey::GreaterThan,
            QueryValue::Value(Into::<Number>::into(value).into()),
        )
    }

    pub fn greater_than_equal(&mut self, value: impl Into<Number>) -> &mut Self {
        self.push(
            QueryKey::GreaterThanEqual,
            QueryValue::Value(Into::<Number>::into(value).into()),
        )
    }

    pub fn less_than(&mut self, value: impl Into<Number>) -> &mut Self {
        self.push(
            QueryKey::LessThan,
            QueryValue::Value(Into::<Number>::into(value).into()),
        )
    }

    pub fn less_than_equal(&mut self, value: impl Into<Number>) -> &mut Self {
        self.push(
            QueryKey::LessThanEqual,
            QueryValue::Value(Into::<Number>::into(value).into()),
        )
    }

    pub fn equals(&mut self, value: impl Into<Value>) -> &mut Self {
        self.push(QueryKey::Equals, QueryValue::Value(value.into()))
    }

    pub fn not_equals(&mut self, value: impl Into<Value>) -> &mut Self {
        self.push(QueryKey::NotEquals, QueryValue::Value(value.into()))
    }

    pub fn in_array(&mut self, value: impl IntoIterator<Item = impl Into<Value>>) -> &mut Self {
        self.push(
            QueryKey::In,
            QueryValue::Value(Value::Array(
                value
                    .into_iter()
                    .map(|v| Into::<Value>::into(v).clone())
                    .collect::<Vec<Value>>(),
            )),
        )
    }

    pub fn not_in_array(&mut self, value: impl IntoIterator<Item = impl Into<Value>>) -> &mut Self {
        self.push(
            QueryKey::NotIn,
            QueryValue::Value(Value::Array(
                value
                    .into_iter()
                    .map(|v| Into::<Value>::into(v))
                    .collect::<Vec<Value>>(),
            )),
        )
    }

    pub fn not(&mut self, value: impl Into<Query>) -> &mut Self {
        self.push(QueryKey::Not, QueryValue::Mapping(value.into()))
    }

    pub fn and(&mut self, cases: impl IntoIterator<Item = impl Into<Query>>) -> &mut Self {
        self.push(
            QueryKey::And,
            QueryValue::Casematch(
                cases
                    .into_iter()
                    .map(|c| Into::<Query>::into(c))
                    .collect::<Vec<Query>>(),
            ),
        )
    }

    pub fn or(&mut self, cases: impl IntoIterator<Item = impl Into<Query>>) -> &mut Self {
        self.push(
            QueryKey::Or,
            QueryValue::Casematch(
                cases
                    .into_iter()
                    .map(|c| Into::<Query>::into(c))
                    .collect::<Vec<Query>>(),
            ),
        )
    }

    pub fn build(&self) -> Self {
        self.clone()
    }
}

fn bson_value(input: &Bson) -> OResult<Value> {
    to_value(input).or_else(|e| {
        Err(OrmoxError::Deserialization {
            error: e.to_string(),
        })
    })
}

fn bson_value_array(input: &Bson) -> OResult<Vec<Value>> {
    to_value(input)
        .or_else(|e| {
            Err(OrmoxError::Deserialization {
                error: e.to_string(),
            })
        })?
        .as_array()
        .ok_or(OrmoxError::Deserialization {
            error: String::from("Expected an array of values"),
        })
        .cloned()
}

fn bson_number(input: &Bson) -> OResult<Number> {
    bson_value(input)?
        .as_number()
        .ok_or(OrmoxError::Deserialization {
            error: String::from("Invalid number"),
        })
        .cloned()
}

fn bson_query(input: &Bson) -> OResult<Query> {
    TryFrom::<bson::Document>::try_from(
        input
            .as_document()
            .ok_or(OrmoxError::Deserialization {
                error: String::from("Expected a document"),
            })?
            .clone(),
    )
}

fn bson_query_array(input: &Bson) -> OResult<Vec<Query>> {
    let mut result: Vec<Query> = Vec::new();
    for item in input.as_array().ok_or(OrmoxError::Deserialization {
        error: String::from("Expected an array of values"),
    })? {
        result.push(bson_query(item)?);
    }
    Ok(result)
}

impl TryFrom<bson::Document> for Query {
    type Error = OrmoxError;
    fn try_from(value: bson::Document) -> Result<Self, Self::Error> {
        let mut result = Query::new();
        for (key, value) in value {
            if key.starts_with("$") {
                match key.as_str() {
                    "$gt" => result.greater_than(bson_number(&value)?),
                    "$lt" => result.less_than(bson_number(&value)?),
                    "$gte" => result.greater_than_equal(bson_number(&value)?),
                    "$lte" => result.less_than_equal(bson_number(&value)?),
                    "$eq" => result.equals(bson_value(&value)?),
                    "$ne" => result.not_equals(bson_value(&value)?),
                    "$in" => result.in_array(bson_value_array(&value)?),
                    "$nin" => result.not_in_array(bson_value_array(&value)?),
                    "$not" => result.not(bson_query(&value)?),
                    "$and" => result.and(bson_query_array(&value)?),
                    "$or" => result.or(bson_query_array(&value)?),
                    op => result.operation(
                        op,
                        if let Bson::Document(subdoc) = value {
                            QueryValue::Mapping(TryFrom::<bson::Document>::try_from(subdoc)?)
                        } else if let Ok(queries) = bson_query_array(&value) {
                            QueryValue::Casematch(queries)
                        } else {
                            QueryValue::Value(bson_value(&value)?)
                        },
                    ),
                };
            } else {
                if let Bson::Document(subdoc) = value {
                    result.subquery(key, Query::try_from(subdoc)?);
                } else {
                    result.field(key, bson_value(&value)?);
                }
            }
        }

        Ok(result)
    }
}

impl TryInto<bson::Document> for Query {
    type Error = OrmoxError;
    fn try_into(self) -> Result<bson::Document, Self::Error> {
        let mut result = bson::Document::new();

        for (key, value) in self.0 {
            match value {
                QueryValue::Value(v) => result.insert(
                    key.to_string(),
                    Bson::try_from(v).or_else(|e| {
                        Err(OrmoxError::Deserialization {
                            error: e.to_string(),
                        })
                    })?,
                ),
                QueryValue::Casematch(queries) => {
                    let mut cases: Vec<Bson> = Vec::new();
                    for q in queries {
                        cases.push(Bson::Document(q.try_into()?));
                    }

                    result.insert(key.to_string(), Bson::Array(cases))
                }
                QueryValue::Mapping(query) => {
                    result.insert(key.to_string(), Bson::Document(query.try_into()?))
                }
            };
        }

        Ok(result)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleQuery(Query);

impl SimpleQuery {
    pub fn new() -> Self {
        Self(Query::new())
    }

    fn q(&mut self) -> &mut Query {
        &mut self.0
    }

    pub fn equals(&mut self, key: impl AsRef<str>, value: impl Into<Value>) -> &mut Self {
        self.q().field(key, value);
        self
    }

    pub fn not_equals(&mut self, key: impl AsRef<str>, value: impl Into<Value>) -> &mut Self {
        self.q()
            .subquery(key, Query::new().not_equals(value).build());
        self
    }

    pub fn less_than(&mut self, key: impl AsRef<str>, value: impl Into<Number>) -> &mut Self {
        self.q()
            .subquery(key, Query::new().less_than(value).build());
        self
    }

    pub fn less_than_equal(&mut self, key: impl AsRef<str>, value: impl Into<Number>) -> &mut Self {
        self.q()
            .subquery(key, Query::new().less_than_equal(value).build());
        self
    }

    pub fn greater_than(&mut self, key: impl AsRef<str>, value: impl Into<Number>) -> &mut Self {
        self.q()
            .subquery(key, Query::new().greater_than(value).build());
        self
    }

    pub fn greater_than_equal(
        &mut self,
        key: impl AsRef<str>,
        value: impl Into<Number>,
    ) -> &mut Self {
        self.q()
            .subquery(key, Query::new().greater_than_equal(value).build());
        self
    }

    pub fn in_array(
        &mut self,
        key: impl AsRef<str>,
        value: impl IntoIterator<Item = impl Into<Value>>,
    ) -> &mut Self {
        self.q().subquery(key, Query::new().in_array(value).build());
        self
    }

    pub fn not_in_array(
        &mut self,
        key: impl AsRef<str>,
        value: impl IntoIterator<Item = impl Into<Value>>,
    ) -> &mut Self {
        self.q()
            .subquery(key, Query::new().not_in_array(value).build());
        self
    }

    pub fn not(&mut self, key: impl AsRef<str>, expr: impl Into<Query>) -> &mut Self {
        self.q().subquery(key, Query::new().not(expr).build());
        self
    }

    pub fn build(&self) -> Query {
        self.0.clone().build()
    }
}

impl From<Query> for SimpleQuery {
    fn from(value: Query) -> Self {
        Self(value)
    }
}

impl From<SimpleQuery> for Query {
    fn from(value: SimpleQuery) -> Self {
        value.0
    }
}
