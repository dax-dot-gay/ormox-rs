use std::sync::{Arc, OnceLock};

pub mod core;
pub mod client;
pub use uuid;
pub use serde;
pub use bson;
pub use thiserror;

pub use {
    core::error::{OResult, OrmoxError},
    core::document::{Document, Index},
    core::driver::{DatabaseDriver, Find, FindBuilder, FindBuilderError, Sorting},
    core::query::{Query, QueryKey, QueryValue, SimpleQuery},
    client::{Client, Collection}
};

pub(crate) static ORMOX: OnceLock<Arc<Client>> = OnceLock::new();