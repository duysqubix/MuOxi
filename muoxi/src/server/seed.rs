//! One-time world seed for kick-the-tires demos.
//!
//! Idempotent: skipped on subsequent boots by looking for an object tagged
//! `(starting-room, system)`. Safe to call on every startup.

use crate::world::WorldApi;
use db::utils::UID;
use serde_json::json;
use std::error::Error;

/// Tag pair marking the starting room so subsequent boots can find it.
pub const STARTING_ROOM_TAG: (&str, &str) = ("starting-room", "system");

/// Apply a minimal demo world. Returns the starting room's UID.
///
/// On first boot, creates:
/// - A `room` named "Limbo" with a `desc` attribute
/// - A `item` named "a polished stone" inside Limbo
/// - A `mob` named "a tired-looking goblin" inside Limbo
///
/// On subsequent boots, returns the existing starting room's UID without
/// touching anything else.
pub async fn seed_world(world: &WorldApi) -> Result<UID, Box<dyn Error + Send + Sync>> {
    let existing = world
        .objects_with_tag(STARTING_ROOM_TAG.0, STARTING_ROOM_TAG.1)
        .await
        .unwrap_or_default();
    if let Some(uid) = existing.first().copied() {
        log::info!("World already seeded; starting room uid={uid}");
        return Ok(uid);
    }

    let room = world.create_object("room", "Limbo", None).await?;
    world
        .set_attribute(
            room.uid,
            "desc",
            json!(
                "You stand in a featureless void. The air feels still and timeless. \
                 A polished stone sits at your feet, and a tired-looking goblin slumps nearby."
            ),
        )
        .await?;
    world
        .add_tag(room.uid, STARTING_ROOM_TAG.0, STARTING_ROOM_TAG.1)
        .await?;

    let _stone = world
        .create_object("item", "a polished stone", Some(room.uid))
        .await?;
    let _goblin = world
        .create_object("mob", "a tired-looking goblin", Some(room.uid))
        .await?;

    log::info!("Seeded world: starting room uid={}", room.uid);
    Ok(room.uid)
}
