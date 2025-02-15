use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

pub trait Document: Serialize + DeserializeOwned + Clone + Debug {
    fn id(&self) -> Uuid;
}
