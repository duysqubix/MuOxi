//!
//! Holds templates and query related functions about clients
//! Clients are the raw representation of connected socket and file
//! descriptors.
//!
use super::schema::clients;
use crate::utils::UID;
use diesel::expression_methods::TextExpressionMethods;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;

/// representation of raw socket for incoming connection
#[derive(Queryable, Insertable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
pub struct Client {
    /// unique id for each client
    pub uid: i64,

    /// ip address where client has connected from
    pub ip: String,

    /// port where client has connected from
    pub port: i32,
}

impl Client {
    /// create new Client
    pub fn new(uid: UID, ip: String, port: i32) -> Self {
        Self { uid, ip, port }
    }
}

/// Wrapper around client json representation object
#[derive(Debug)]
pub struct ClientHashMap(pub HashMap<UID, Client>);

/// Wrapper around a vector of clients objects
#[derive(Debug)]
pub struct ClientVector(pub Vec<Client>);

impl ClientVector {
    /// returns an empty initialized vector of clients
    pub fn empty() -> Self {
        Self(vec![])
    }

    /// returns the length of current vector
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<ClientHashMap> for ClientVector {
    fn from(hmap: ClientHashMap) -> Self {
        ClientVector(hmap.0.iter().map(|(_, c)| c.clone()).collect())
    }
}

/// Holds utilities to maniuplate the Client table in the database
pub struct ClientHandler;
impl ClientHandler {
    /// Attempts to insert a new client with UID, if there is a conflic,
    /// it will update the record.
    pub fn upsert(&self, conn: &PgConnection, new_client: &Client) -> QueryResult<Client> {
        diesel::insert_into(clients::table)
            .values(new_client)
            .on_conflict(clients::uid)
            .do_update()
            .set(new_client)
            .get_result(conn)
    }

    /// Attempts to insert a new client with UID, if there is a conflic,
    /// it will update the record. Doesn't work quite as expected.. Followed the
    /// guides from [here](https://stackoverflow.com/questions/47626047/execute-an-insert-or-update-using-diesel)
    /// but doesn't seem to actually `set` the excluded value where the conflict happened..
    pub fn upsert_batch(&self, conn: &PgConnection, clients: ClientVector) -> QueryResult<()> {
        diesel::insert_into(clients::table)
            .values(&clients.0)
            .on_conflict(clients::uid)
            .do_update()
            .set(clients::uid.eq(excluded(clients::uid)))
            .execute(conn)?;
        Ok(())
    }

    /// Permanently remove record from table, by UID
    pub fn remove_uid(&self, conn: &PgConnection, id: UID) -> QueryResult<usize> {
        use self::clients::dsl;

        diesel::delete(dsl::clients.filter(dsl::uid.eq(id))).execute(conn)
    }

    /// Remove a list of UIDS from db by suppling a vec of UIDs
    /// *careful* if supplied vector is empty, it will remove all records in table
    pub fn remove_uids(&self, conn: &PgConnection, uids: Vec<UID>) -> QueryResult<usize> {
        use self::clients::dsl;

        let mut deleted = 0;

        if uids.len() == 0 {
            return diesel::delete(dsl::clients).execute(conn);
        }

        for uid in uids.iter() {
            deleted += diesel::delete(dsl::clients.filter(dsl::uid.eq(uid))).execute(conn)?;
        }
        Ok(deleted)
    }

    /// checks to see if UID of client exists within database
    pub fn uid_exists(&self, conn: &PgConnection, id: UID) -> bool {
        let mut exists: bool = false;
        let record = self.get_uid(conn, id).unwrap_or(ClientVector::empty());

        if record.len() > 0 {
            exists = true;
        }

        exists
    }

    /// Get single UID from db, if UID doesn't exist it will
    /// return an empty vector
    pub fn get_uid(&self, conn: &PgConnection, id: UID) -> QueryResult<ClientVector> {
        use self::clients::dsl;

        let record = dsl::clients.filter(dsl::uid.eq(id)).load::<Client>(conn)?;
        Ok(ClientVector(record))
    }

    /// Retrieve a list of UIDS from db by suppling a vec of UIDs
    /// if an empty set of uids is supplied, it will retrieve all records in table
    pub fn get_uids(&self, conn: &PgConnection, uids: Vec<UID>) -> QueryResult<ClientVector> {
        use self::clients::dsl;

        let mut results: Vec<Client> = Vec::new();

        if uids.len() == 0 {
            let all_records = dsl::clients.load::<Client>(conn)?;
            return Ok(ClientVector(all_records));
        }

        for uid in uids.iter() {
            let record = dsl::clients.filter(dsl::uid.eq(uid)).load::<Client>(conn)?;

            if let Some(client) = record.first() {
                results.push(client.clone());
            } else {
                println!("Couldn't find record with uid: `{}`", uid);
            }
        }
        Ok(ClientVector(results))
    }

    /// Get a range of UIDs
    /// Note that UIDS that do not exists will be logged, but will not
    /// show up in the returned vector
    pub fn get_uids_range(
        &self,
        conn: &PgConnection,
        from: UID,
        to: UID,
    ) -> QueryResult<ClientVector> {
        let uid_range: Vec<UID> = (from..to).collect();
        self.get_uids(conn, uid_range)
    }

    /// retrieve records with exact IP address
    pub fn get_ip_exact<'a>(&self, conn: &PgConnection, ip: &'a str) -> QueryResult<ClientVector> {
        use self::clients::dsl;

        let record = dsl::clients
            .filter(dsl::ip.eq(ip.to_string()))
            .load::<Client>(conn)?;

        Ok(ClientVector(record))
    }

    /// retrieve a vector of ip address with the appropriate
    /// match pattern provided
    ///
    /// ```rust, ignore
    /// let pattern = format!("%{}%", "168.0");
    ///
    /// // vector of Clients objects with ips matching `*168.0*`
    /// let results = ClientHandler::get_ip_like(&conn, pattern);
    ///
    /// ```
    pub fn get_ip_like<'a>(
        &self,
        conn: &PgConnection,
        pattern_match: String,
    ) -> QueryResult<ClientVector> {
        use self::clients::dsl;

        let records = dsl::clients
            .filter(dsl::ip.like(pattern_match))
            .load::<Client>(conn)?;

        Ok(ClientVector(records))
    }
}
