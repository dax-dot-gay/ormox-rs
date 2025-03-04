pub use ormox_core::{
    client::{Client, Collection, self},
    core::{
        document::{Document, Index},
        driver::{DatabaseDriver, Find, Sorting},
        error::OrmoxError as Error,
        query::{Query, QueryKey, QueryValue, SimpleQuery},
        self
    },
};

pub use ormox_core;

#[cfg(feature = "derive")]
pub use ormox_derive::{ormox_document, Document};

pub mod drivers {
    #[cfg(feature = "polodb")]
    pub use ormox_driver_polodb::PoloDriver;

    #[cfg(feature = "mongodb")]
    pub use ormox_driver_mongodb::MongoDriver;
}