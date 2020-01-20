#![deny(missing_docs)]

//!
//! Holds all serializable structures that maps to postgres tables defined
//! in migrations folder
//!

use crate::utils::UID;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::iter::FromIterator;

/// A representation a vector of records from database
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
pub struct RecordHashMap<T>(pub HashMap<UID, T>);

impl<T: Clone> From<RecordHashMap<T>> for RecordVector<T> {
    fn from(hmap: RecordHashMap<T>) -> Self {
        let v = hmap.0.values().cloned().collect();
        RecordVector(v)
    }
}

/// a trait that handles interaction with the database
pub trait DatabaseHandler<T> {
    /// attempt to insert record, on conflict
    /// it will update the record
    fn upsert(&self, conn: &PgConnection, record: T) -> QueryResult<T>;

    /// attempt to insert record, on conflict will return None
    fn insert(&self, conn: &PgConnection, record: T) -> QueryResult<Option<T>>;

    /// removes record from database
    fn remove(&self, conn: &PgConnection, uid: UID) -> QueryResult<usize>;

    /// retrieves a record from database using UID
    fn get(&self, conn: &PgConnection, id: UID) -> QueryResult<RecordVector<T>>;

    /// retrieves a list of records from a list of UIDS for object T
    fn get_batch(&self, conn: &PgConnection, uids: Vec<UID>) -> QueryResult<RecordVector<T>>;

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
        pub characters: Vec<UID>,
    }

    /// Holds utilities to CRUD the Account table in the database
    pub struct AccountHandler;

    /// Builder struct that creates Accounts with friendly API
    pub struct AccountBuilder {
        name: Option<String>,

        email: Option<String>,
        password: Option<String>,
    }

    impl<'a> AccountBuilder {
        /// create with name
        pub fn with_name(&mut self, name: &'a str) -> &mut Self {
            self.name = Some(String::from(name));
            self
        }

        /// create with email
        pub fn with_email(&mut self, email: &'a str) -> &mut Self {
            self.email = Some(String::from(email));
            self
        }

        /// create with password
        pub fn with_password(&mut self, pass: &'a str) -> &mut Self {
            self.password = Some(String::from(pass));
            self
        }

        /// consume self and create Account
        pub fn create(self) -> Account {
            Account {
                uid: gen_uid(),
                name: self.name.unwrap(),
                email: self.email.unwrap(),
                password: self.password.unwrap(),
                characters: Vec::new(),
            }
        }
    }
}

/// holds db information regarding playable characters
pub mod character {

    use super::super::schema::accounts;
    use crate::utils::{gen_uid, UID};

    /// representation of the actual playable character
    // #[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
    pub struct Character {
        /// unique id for each account
        pub uid: UID,
        /// name of account
        pub name: String,
    }

    /// Holds utilities to CRUD the Character table in the database
    pub struct CharacterHandler;
}
