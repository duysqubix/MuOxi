#![deny(missing_docs)]

//!
//! Holds all serializable structures that maps to postgres tables defined
//! in migrations folder
//!

use crate::utils::UID;
use diesel::expression_methods::TextExpressionMethods;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::iter::FromIterator;

/// A representation a vector of records from database
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

    /// add element to vector
    pub fn add(&mut self, elem: T) {
        self.0.push(elem);
    }
}

impl<T> FromIterator<T> for RecordVector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = RecordVector::empty();

        for i in iter {
            c.add(i)
        }
        c
    }
}

/// A representation of json object
#[derive(Debug, Clone)]
pub struct RecordHashMap<T>(pub HashMap<UID, T>);

impl<T: Clone> From<RecordHashMap<T>> for RecordVector<T> {
    fn from(hmap: RecordHashMap<T>) -> Self {
        let v = hmap.0.values().cloned().collect();
        RecordVector(v)
    }
}

/// a trait that handles interaction with the database
pub trait DatabaseHandlerExt<T> {
    /// attempt to insert record, on conflict
    /// it will update the record
    fn upsert(&self, conn: &PgConnection, record: &T) -> QueryResult<T>;

    /// attempt to insert record, on conflict will return None
    fn insert(&self, conn: &PgConnection, record: &T) -> Option<T>;

    /// removes record from database
    fn remove(&self, conn: &PgConnection, uid: UID) -> QueryResult<usize>;

    /// retrieves a record from database using UID
    fn get(&self, conn: &PgConnection, id: UID) -> QueryResult<RecordVector<T>>;

    /// retrieves a list of records from a list of UIDS for object T
    fn get_batch(&self, conn: &PgConnection, uids: Vec<UID>) -> QueryResult<RecordVector<T>>;

    /// Get a range of UIDs
    fn get_range(&self, conn: &PgConnection, from: UID, to: UID) -> QueryResult<RecordVector<T>> {
        let uid_range: Vec<UID> = (from..to).collect();
        self.get_batch(conn, uid_range)
    }

    /// checks to see if a records exists
    fn exists(&self, conn: &PgConnection, uid: UID) -> bool {
        let mut exist = false;
        let record = self.get(conn, uid).unwrap_or(RecordVector::empty());

        if record.len() > 0 {
            exist = true
        }

        exist
    }
}

/// holds db related information about accounts
pub mod account {
    use super::super::schema::accounts;
    use super::*;
    use crate::utils::{gen_uid, UID};

    /// representation of accounts of clients, holds all characters and meta
    /// information.
    #[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
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

    /// Holds utilities to CRUD the Account table in the database
    pub struct AccountHandler;

    impl DatabaseHandlerExt<Account> for AccountHandler {
        fn upsert(&self, conn: &PgConnection, record: &Account) -> QueryResult<Account> {
            diesel::insert_into(accounts::table)
                .values(record)
                .on_conflict(accounts::uid)
                .do_update()
                .set(record)
                .get_result(conn)
        }

        fn insert(&self, conn: &PgConnection, record: &Account) -> Option<Account> {
            let record_result = diesel::insert_into(accounts::table)
                .values(record)
                .get_result(conn);

            match record_result {
                Ok(result) => Some(result),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            }
        }

        fn remove(&self, conn: &PgConnection, uid: UID) -> QueryResult<usize> {
            use self::accounts::dsl;
            diesel::delete(dsl::accounts.filter(dsl::uid.eq(uid))).execute(conn)
        }

        fn get(&self, conn: &PgConnection, uid: UID) -> QueryResult<RecordVector<Account>> {
            use self::accounts::dsl;
            let record = dsl::accounts
                .filter(dsl::uid.eq(uid))
                .load::<Account>(conn)?;
            Ok(RecordVector(record))
        }

        fn get_batch(
            &self,
            conn: &PgConnection,
            uids: Vec<UID>,
        ) -> QueryResult<RecordVector<Account>> {
            use self::accounts::dsl;

            let mut results: Vec<Account> = Vec::new();

            if uids.len() == 0 {
                let all_records = dsl::accounts.load::<Account>(conn)?;
                return Ok(RecordVector(all_records));
            }

            for uid in uids.iter() {
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

/// holds db information regarding playable characters
pub mod character {
    use super::super::schema::characters;
    use super::*;
    use crate::utils::{gen_uid, UID};

    /// representation of the actual playable character
    #[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
    pub struct Character {
        /// unique id for each chatacter
        pub uid: UID,

        /// uid for account associated with character
        pub account: UID,

        /// name of character
        pub name: String,
    }

    /// Holds utilities to CRUD the Character table in the database
    pub struct CharacterHandler;

    impl DatabaseHandlerExt<Character> for CharacterHandler {
        fn upsert(&self, conn: &PgConnection, record: &Character) -> QueryResult<Character> {
            diesel::insert_into(characters::table)
                .values(record)
                .on_conflict(characters::uid)
                .do_update()
                .set(record)
                .get_result(conn)
        }

        fn insert(&self, conn: &PgConnection, record: &Character) -> Option<Character> {
            let record_result = diesel::insert_into(characters::table)
                .values(record)
                .get_result(conn);

            match record_result {
                Ok(result) => Some(result),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            }
        }

        fn remove(&self, conn: &PgConnection, uid: UID) -> QueryResult<usize> {
            use self::characters::dsl;
            diesel::delete(dsl::characters.filter(dsl::uid.eq(uid))).execute(conn)
        }

        fn get(&self, conn: &PgConnection, uid: UID) -> QueryResult<RecordVector<Character>> {
            use self::characters::dsl;
            let record = dsl::characters
                .filter(dsl::uid.eq(uid))
                .load::<Character>(conn)?;
            Ok(RecordVector(record))
        }

        fn get_batch(
            &self,
            conn: &PgConnection,
            uids: Vec<UID>,
        ) -> QueryResult<RecordVector<Character>> {
            use self::characters::dsl;
            let mut results: Vec<Character> = Vec::new();

            if uids.len() == 0 {
                let all_records = dsl::characters.load::<Character>(conn)?;
                return Ok(RecordVector(all_records));
            }

            for uid in uids.iter() {
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
