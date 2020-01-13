#![allow(missing_docs)]
table! {
    clients (uid) {
        uid -> Int8,
        ip -> Varchar,
        port -> Int4,
    }
}
