use bson::DecoderResult;
use bson::{Bson, Document};
use serde::{Deserialize, Serialize};

pub enum FilterOn {
    UID,
    NAME,
}

pub trait MongoDocument {
    fn name(&self) -> String;
    fn uid(&self) -> i64;
}

pub fn to_object<'de, T: Serialize + Deserialize<'de> + MongoDocument>(
    doc: Document,
) -> DecoderResult<T> {
    bson::from_bson(Bson::Document(doc))
}
