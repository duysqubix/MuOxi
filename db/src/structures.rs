#![deny(missing_docs)]

//! Holds all serializable structures that map to the Postgres tables
//! defined in the `migrations/` folder.

use crate::utils::UID;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::iter::FromIterator;

/// A vector of records from the database.
#[derive(Debug, Clone)]
pub struct RecordVector<T>(pub Vec<T>);

impl<T> RecordVector<T> {
    /// returns an empty initialized vector of records
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// returns length of the current vector
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// returns true if vector is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// add element to vector
    pub fn add(&mut self, elem: T) {
        self.0.push(elem);
    }
}

impl<T> FromIterator<T> for RecordVector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = RecordVector::empty();
        for i in iter {
            c.add(i);
        }
        c
    }
}

/// A representation of a JSON-style object as a UID -> record map.
#[derive(Debug, Clone)]
pub struct RecordHashMap<T>(pub HashMap<UID, T>);

impl<T: Clone> From<RecordHashMap<T>> for RecordVector<T> {
    fn from(hmap: RecordHashMap<T>) -> Self {
        let v = hmap.0.values().cloned().collect();
        RecordVector(v)
    }
}

/// CRUD contract for any ORM record stored in this database.
pub trait DatabaseHandlerExt<T> {
    /// Insert a record; on UID conflict, update the existing record.
    fn upsert(&self, conn: &mut PgConnection, record: &T) -> QueryResult<T>;

    /// Insert a record; on conflict return `None`.
    fn insert(&self, conn: &mut PgConnection, record: &T) -> Option<T>;

    /// Delete a record by UID. Returns rows affected.
    fn remove(&self, conn: &mut PgConnection, uid: UID) -> QueryResult<usize>;

    /// Retrieve the record for a single UID (vec because schema doesn't enforce uniqueness on this path).
    fn get(&self, conn: &mut PgConnection, id: UID) -> QueryResult<RecordVector<T>>;

    /// Retrieve a list of records by UIDs. Empty input returns ALL records.
    fn get_batch(&self, conn: &mut PgConnection, uids: Vec<UID>) -> QueryResult<RecordVector<T>>;

    /// Retrieve a contiguous range of UIDs.
    fn get_range(
        &self,
        conn: &mut PgConnection,
        from: UID,
        to: UID,
    ) -> QueryResult<RecordVector<T>> {
        let uid_range: Vec<UID> = (from..to).collect();
        self.get_batch(conn, uid_range)
    }

    /// Check whether a record exists.
    fn exists(&self, conn: &mut PgConnection, uid: UID) -> bool {
        let record = self.get(conn, uid).unwrap_or_else(|_| RecordVector::empty());
        !record.is_empty()
    }
}

/// db related information about accounts
pub mod account {
    use super::super::schema::accounts;
    use super::*;

    /// Representation of an account, holding all of its characters and metadata.
    #[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
    #[diesel(table_name = accounts)]
    pub struct Account {
        /// unique id for each account
        pub uid: UID,
        /// name of account
        pub name: String,
        /// password for account
        pub password: String,
        /// email associated with account
        pub email: String,
        /// Characters stored as a vector of the character UID numbers
        pub characters: Option<Vec<i64>>,
    }

    /// CRUD utilities for the Accounts table.
    pub struct AccountHandler;

    impl DatabaseHandlerExt<Account> for AccountHandler {
        fn upsert(&self, conn: &mut PgConnection, record: &Account) -> QueryResult<Account> {
            diesel::insert_into(accounts::table)
                .values(record)
                .on_conflict(accounts::uid)
                .do_update()
                .set(record)
                .get_result(conn)
        }

        fn insert(&self, conn: &mut PgConnection, record: &Account) -> Option<Account> {
            match diesel::insert_into(accounts::table)
                .values(record)
                .get_result(conn)
            {
                Ok(result) => Some(result),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            }
        }

        fn remove(&self, conn: &mut PgConnection, uid: UID) -> QueryResult<usize> {
            use self::accounts::dsl;
            diesel::delete(dsl::accounts.filter(dsl::uid.eq(uid))).execute(conn)
        }

        fn get(&self, conn: &mut PgConnection, uid: UID) -> QueryResult<RecordVector<Account>> {
            use self::accounts::dsl;
            let record = dsl::accounts
                .filter(dsl::uid.eq(uid))
                .load::<Account>(conn)?;
            Ok(RecordVector(record))
        }

        fn get_batch(
            &self,
            conn: &mut PgConnection,
            uids: Vec<UID>,
        ) -> QueryResult<RecordVector<Account>> {
            use self::accounts::dsl;

            if uids.is_empty() {
                return Ok(RecordVector(dsl::accounts.load::<Account>(conn)?));
            }

            let mut results: Vec<Account> = Vec::new();
            for uid in &uids {
                let record = dsl::accounts
                    .filter(dsl::uid.eq(uid))
                    .load::<Account>(conn)?;
                if let Some(acct) = record.first() {
                    results.push(acct.clone());
                } else {
                    println!("Couldn't find record with uid: {}", uid);
                }
            }
            Ok(RecordVector(results))
        }
    }
}

/// db related information regarding playable characters
pub mod character {
    use super::super::schema::characters;
    use super::*;

    /// Representation of a playable character.
    #[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
    #[diesel(table_name = characters)]
    pub struct Character {
        /// unique id for each chatacter
        pub uid: UID,
        /// uid for account associated with character
        pub account: UID,
        /// name of character
        pub name: String,
    }

    /// CRUD utilities for the Characters table.
    pub struct CharacterHandler;

    impl DatabaseHandlerExt<Character> for CharacterHandler {
        fn upsert(&self, conn: &mut PgConnection, record: &Character) -> QueryResult<Character> {
            diesel::insert_into(characters::table)
                .values(record)
                .on_conflict(characters::uid)
                .do_update()
                .set(record)
                .get_result(conn)
        }

        fn insert(&self, conn: &mut PgConnection, record: &Character) -> Option<Character> {
            match diesel::insert_into(characters::table)
                .values(record)
                .get_result(conn)
            {
                Ok(result) => Some(result),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            }
        }

        fn remove(&self, conn: &mut PgConnection, uid: UID) -> QueryResult<usize> {
            use self::characters::dsl;
            diesel::delete(dsl::characters.filter(dsl::uid.eq(uid))).execute(conn)
        }

        fn get(&self, conn: &mut PgConnection, uid: UID) -> QueryResult<RecordVector<Character>> {
            use self::characters::dsl;
            let record = dsl::characters
                .filter(dsl::uid.eq(uid))
                .load::<Character>(conn)?;
            Ok(RecordVector(record))
        }

        fn get_batch(
            &self,
            conn: &mut PgConnection,
            uids: Vec<UID>,
        ) -> QueryResult<RecordVector<Character>> {
            use self::characters::dsl;

            if uids.is_empty() {
                return Ok(RecordVector(dsl::characters.load::<Character>(conn)?));
            }

            let mut results: Vec<Character> = Vec::new();
            for uid in &uids {
                let record = dsl::characters
                    .filter(dsl::uid.eq(uid))
                    .load::<Character>(conn)?;
                if let Some(character) = record.first() {
                    results.push(character.clone());
                } else {
                    println!("Couldn't find record with uid: {}", uid);
                }
            }
            Ok(RecordVector(results))
        }
    }
}
