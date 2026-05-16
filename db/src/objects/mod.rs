//! Generic in-world object model.
//!
//! `Object` is the universal entity row (rooms, items, mobs, characters,
//! exits, downstream-defined types). `ObjectAttribute` is a freeform per-object
//! key/value JSON bag. `ObjectTag` is a per-object label with optional category.
//! `CharacterAccount` links character objects to login accounts.
//! `Script` stores persistent scheduled jobs.
//!
//! Engine code should go through the repository structs (`ObjectRepo`,
//! `AttributeRepo`, `TagRepo`, `CharacterAccountRepo`, `ScriptRepo`) and not
//! import `diesel::*` directly.

pub mod attribute;
pub mod character_account;
pub mod object;
pub mod script;
pub mod tag;

pub use attribute::{AttributeRepo, ObjectAttribute};
pub use character_account::{CharacterAccount, CharacterAccountRepo};
pub use object::{NewObject, Object, ObjectRepo};
pub use script::{NewScript, Script, ScriptRepo};
pub use tag::{ObjectTag, TagRepo};
