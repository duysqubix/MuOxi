use bson::DecoderResult;
use bson::{Bson, Document};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub type JsonDecoderResult<T> = Result<T, serde_json::error::Error>;
pub type UID = u64;
pub enum FilterOn {
    UID,
    NAME,
}

pub trait MongoDocument {
    fn name(&self) -> String;
    fn uid(&self) -> UID;
}

///
/// Creates a unique 8 byte address first 4 bytes is timestamp
/// since UNIX_EPOCH and the last 8 bytes are randomly
/// generated values
///
pub fn gen_uid() -> UID {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime is before UNIX_EPOCH");

    let timestamp = now.as_secs() as u64;
    let id = rand::thread_rng().gen_range(0, 0xFF_FF_FF_FF as u64);

    timestamp + id
}

pub fn bson_to_object<'de, T: Serialize + Deserialize<'de> + MongoDocument>(
    doc: Document,
) -> DecoderResult<T> {
    bson::from_bson(Bson::Document(doc))
}

pub fn json_to_object<T: Serialize + DeserializeOwned>(
    val: serde_json::Value,
) -> JsonDecoderResult<T> {
    serde_json::from_value(val)
}
