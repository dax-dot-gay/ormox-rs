use std::sync::{Arc, OnceLock};

use client::Client;

pub mod core;
pub mod client;

pub(crate) static ORMOX: OnceLock<Arc<Client>> = OnceLock::new();