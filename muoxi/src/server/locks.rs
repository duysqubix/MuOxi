//! Minimal lock-expression evaluator.
//!
//! v0.1 supports three expression forms:
//! * `all()` — always allow
//! * `false` — never allow
//! * `perm(<name>)` — actor must have tag `(name, "permission")` on their character object
//!
//! Future versions will expand this into a small DSL with `and`/`or`/`not`,
//! `id(<uid>)`, `holds(<uid>)`, etc.

use crate::world::WorldApi;
use db::utils::UID;

/// Evaluate a lock expression for an actor against the world.
///
/// Returns `true` if the actor is allowed, `false` otherwise. Database errors
/// are conservatively treated as "deny".
pub async fn check(world: &WorldApi, expr: &str, actor: Option<UID>) -> bool {
    let trimmed = expr.trim();
    if trimmed == "all()" {
        return true;
    }
    if trimmed == "false" {
        return false;
    }
    if let Some(name) = trimmed
        .strip_prefix("perm(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let name = name.trim().trim_matches('"');
        let Some(uid) = actor else {
            return false;
        };
        return world.has_tag(uid, name, "permission").await.unwrap_or(false);
    }
    false
}
