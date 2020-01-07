//!
//! Holds all the different models and templates that represent MUD objects
//! within the database.
//!

#[derive(Queryable)]
pub struct Account {
    pub uid: i64,
    pub name: String,
}
