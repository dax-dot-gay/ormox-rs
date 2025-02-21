use std::error::Error;

use ormox::{drivers::PoloDriver, ormox_core::bson::doc, ormox_document, Client, Document};

#[ormox_document(collection = "test", id_field = "id", id_alias = "_id")]
pub struct User {
    #[index]
    pub name: String,
    pub age: i64,

    #[serde(default)]
    pub nickname: Option<String>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::create_global(PoloDriver::new("test.db")?);
    let user = User::create(None, "Test User", 27, None);
    user.save().await?;
    for d in client.collection::<User>().all(None).await? {
        println!("{:?}", d.id());
    }
    println!("{:?}", client.collection::<User>().get(user.id().to_string()).await?.name);

    Ok(())
}
