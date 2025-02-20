use std::sync::{Arc, OnceLock};

use client::Client;

pub mod core;
pub mod client;
pub use uuid;
pub use serde;
pub use bson;

pub(crate) static ORMOX: OnceLock<Arc<Client>> = OnceLock::new();