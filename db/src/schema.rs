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
    characters (uid) {
        uid -> BigInt,
        account_uid -> BigInt,
        name -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    account_characters (account_uid, character_uid) {
        account_uid -> BigInt,
        character_uid -> BigInt,
        ordinal -> Integer,
    }
}

diesel::joinable!(characters -> accounts (account_uid));
diesel::joinable!(account_characters -> accounts (account_uid));
diesel::joinable!(account_characters -> characters (character_uid));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    characters,
    account_characters,
);
