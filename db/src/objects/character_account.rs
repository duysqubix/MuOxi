//! Link table between character objects (`objects.type_key = 'character'`)
//! and login accounts. Replaces the old `characters` + `account_characters` pair.

use crate::conn::Conn;
use crate::schema::character_accounts;
use crate::utils::UID;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A link row.
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = character_accounts)]
pub struct CharacterAccount {
    /// uid of the character object (must reference `objects.uid` where `type_key = 'character'`)
    pub object_uid: UID,
    /// owning account uid
    pub account_uid: UID,
    /// 0-indexed position in this account's character list
    pub ordinal: i32,
}

/// CRUD on the `character_accounts` table.
pub struct CharacterAccountRepo;

impl CharacterAccountRepo {
    /// Link a character object to an account at the next available ordinal.
    pub fn link(
        &self,
        conn: &mut Conn,
        object_uid: UID,
        account_uid: UID,
    ) -> QueryResult<CharacterAccount> {
        let next_ordinal: i32 = character_accounts::table
            .filter(character_accounts::account_uid.eq(account_uid))
            .select(diesel::dsl::max(character_accounts::ordinal))
            .first::<Option<i32>>(conn)?
            .unwrap_or(-1)
            + 1;

        let row = CharacterAccount {
            object_uid,
            account_uid,
            ordinal: next_ordinal,
        };
        diesel::insert_into(character_accounts::table)
            .values(&row)
            .execute(conn)?;
        Ok(row)
    }

    /// Unlink a character (does NOT delete the underlying object — caller decides).
    pub fn unlink(&self, conn: &mut Conn, object_uid: UID) -> QueryResult<usize> {
        diesel::delete(
            character_accounts::table.filter(character_accounts::object_uid.eq(object_uid)),
        )
        .execute(conn)
    }

    /// All characters owned by `account_uid`, ordered by `ordinal`.
    pub fn list_for_account(
        &self,
        conn: &mut Conn,
        account_uid: UID,
    ) -> QueryResult<Vec<CharacterAccount>> {
        character_accounts::table
            .filter(character_accounts::account_uid.eq(account_uid))
            .order(character_accounts::ordinal.asc())
            .load::<CharacterAccount>(conn)
    }

    /// The account that owns a given character object, if any.
    pub fn owner_of(
        &self,
        conn: &mut Conn,
        object_uid: UID,
    ) -> QueryResult<Option<CharacterAccount>> {
        character_accounts::table
            .filter(character_accounts::object_uid.eq(object_uid))
            .first::<CharacterAccount>(conn)
            .optional()
    }
}
