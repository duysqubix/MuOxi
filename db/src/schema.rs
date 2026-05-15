diesel::table! {
    accounts (uid) {
        uid -> BigInt,
        name -> Text,
        password_hash -> Text,
        email -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    objects (uid) {
        uid -> BigInt,
        type_key -> Text,
        name -> Text,
        location_uid -> Nullable<BigInt>,
        created_at -> BigInt,
    }
}

diesel::table! {
    object_attributes (object_uid, key) {
        object_uid -> BigInt,
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    object_tags (object_uid, key, category) {
        object_uid -> BigInt,
        key -> Text,
        category -> Text,
    }
}

diesel::table! {
    character_accounts (object_uid) {
        object_uid -> BigInt,
        account_uid -> BigInt,
        ordinal -> Integer,
    }
}

diesel::joinable!(object_attributes -> objects (object_uid));
diesel::joinable!(object_tags -> objects (object_uid));
diesel::joinable!(character_accounts -> objects (object_uid));
diesel::joinable!(character_accounts -> accounts (account_uid));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    objects,
    object_attributes,
    object_tags,
    character_accounts,
);
