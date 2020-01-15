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
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(accounts, posts,);
