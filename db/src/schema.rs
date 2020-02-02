table! {
    accounts (uid) {
        uid -> Int8,
        name -> Varchar,
        password -> Varchar,
        email -> Varchar,
        characters -> Nullable<Array<Int8>>,
    }
}

table! {
    characters (uid) {
        uid -> Int8,
        account -> Int8,
        name -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    accounts,
    characters,
);
