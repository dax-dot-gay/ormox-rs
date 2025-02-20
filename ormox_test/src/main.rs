use ormox::{ormox_core::bson::doc, ormox_document, Document};

#[ormox_document(collection = "test", id_field = "id", id_alias = "beans")]
pub struct TestStruct {
    #[index]
    pub test: String,
}

fn main() {
    let test = TestStruct::parse(doc! {"test": "beans"}, None).unwrap();
    println!("{}: {}", test.id(), test.test);
}
