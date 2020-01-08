//!
//! Holds templates and query related functions about clients
//! Clients are the raw representation of connected socket and file
//! descriptors.
//!
use super::schema::clients;
use diesel::expression_methods::TextExpressionMethods;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::default::Default;
use std::fmt::Debug;
use std::string::ToString;

#[derive(Queryable, Insertable, Debug, AsChangeset, Clone)]
pub struct Client {
    pub uid: i64,
    pub ip: String,
    pub port: i16,
}

#[derive(Debug)]
pub struct FilterCriteria<'a, T: ToString>(pub &'a str, pub T, pub i64);

#[derive(Debug)]
pub enum ClientFilterOn {
    UID(clients::dsl::uid),
    IP(clients::dsl::ip),
    Port(clients::dsl::port),
    NULL,
}

impl ClientFilterOn {
    pub fn new<'a>(field: &'a str) -> Self {
        match field.to_lowercase().as_str() {
            "uid" => Self::UID(clients::dsl::uid),
            "ip" => Self::IP(clients::dsl::ip),
            "port" => Self::Port(clients::dsl::port),
            _ => Self::NULL,
        }
    }

    fn delete<'a>(&self, conn: &PgConnection, criteria: &'a str) -> QueryResult<usize> {
        use clients::dsl;

        match self {
            ClientFilterOn::UID(uid) => {
                let criteria = criteria.parse::<i64>().unwrap_or_else(|e| {
                    println!("{:?}", e);
                    0
                });
                diesel::delete(dsl::clients.filter(uid.eq(criteria))).execute(conn)
            }
            ClientFilterOn::IP(ip) => {
                let str_pattern = format!("{}", criteria);
                diesel::delete(dsl::clients.filter(ip.like(str_pattern))).execute(conn)
            }
            ClientFilterOn::Port(port) => {
                let criteria = criteria.parse::<i16>().unwrap_or_else(|e| {
                    println!("{:?}", e);
                    0
                });
                diesel::delete(dsl::clients.filter(port.eq(criteria))).execute(conn)
            }
            ClientFilterOn::NULL => Ok(0),
        }
    }
    fn get<'a>(
        &self,
        conn: &PgConnection,
        criteria: &'a str,
        limit: i64,
    ) -> QueryResult<Vec<Client>> {
        match self {
            ClientFilterOn::UID(uid) => {
                let criteria = criteria.parse::<i64>().unwrap_or_else(|e| {
                    println!("{:?}", e);
                    0
                });
                clients::dsl::clients
                    .filter(uid.eq(criteria))
                    .limit(limit)
                    .load::<Client>(conn)
            }
            ClientFilterOn::IP(ip) => {
                let str_pattern = format!("{}", criteria);
                clients::dsl::clients
                    .filter(ip.like(str_pattern))
                    .limit(limit)
                    .load::<Client>(conn)
            }
            ClientFilterOn::Port(port) => {
                let criteria = criteria.parse::<i16>().unwrap_or_else(|e| {
                    println!("{:?}", e);
                    0
                });

                clients::dsl::clients
                    .filter(port.eq(criteria))
                    .limit(limit)
                    .load::<Client>(conn)
            }
            ClientFilterOn::NULL => clients::dsl::clients.limit(limit).load::<Client>(conn),
        }
    }
}

pub struct ClientHandler;
impl ClientHandler {
    /// Attempts to insert a new client with UID, if there is a conflic,
    /// it will update the record.
    pub fn upsert(&self, conn: &PgConnection, new_client: Client) -> QueryResult<Client> {
        diesel::insert_into(clients::table)
            .values(&new_client)
            .on_conflict(clients::uid)
            .do_update()
            .set(&new_client)
            .get_result(conn)
    }

    pub fn delete<'a, T: ToString + Debug>(
        &self,
        conn: &PgConnection,
        client: Client,
        filter_criteria: FilterCriteria<T>,
    ) -> QueryResult<usize> {
        let FilterCriteria(on, criteria, _) = filter_criteria;

        ClientFilterOn::new(on).delete(conn, criteria.to_string().as_str())
    }

    pub fn get<'a, T: ToString + Debug>(
        &self,
        conn: &PgConnection,
        filter_criteria: FilterCriteria<T>,
    ) -> QueryResult<Vec<Client>> {
        let FilterCriteria(on, criteria, limit) = filter_criteria;

        ClientFilterOn::new(on).get(conn, criteria.to_string().as_str(), limit)
    }
}
