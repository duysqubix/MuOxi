use rand::Rng;
use std::time::SystemTime;

pub type UID = i64;

///
/// Creates a unique 8 byte address first 4 bytes is timestamp
/// since UNIX_EPOCH and the last 8 bytes are randomly
/// generated values
///
pub fn gen_uid() -> UID {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime is before UNIX_EPOCH");

    let timestamp = now.as_secs() as i64;
    let id = rand::thread_rng().gen_range(0, 0xFF_FF_FF_FF as i64);

    ((timestamp << 32) | id) as UID
}
