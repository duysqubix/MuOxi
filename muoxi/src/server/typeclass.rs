//! `TypeClass` — defines an in-world type's defaults: name prefix, default
//! attributes, default tags, default command set, and lock map.

use crate::prelude::Command;
use std::collections::HashMap;
use std::sync::Arc;

/// Pluggable definition of an in-world type.
pub trait TypeClass: Send + Sync {
    /// Stable string identifier — also stored in `objects.type_key`.
    fn key(&self) -> &'static str;

    /// Friendly description (admin diagnostics).
    fn description(&self) -> &'static str {
        ""
    }

    /// Default attribute values applied at creation time.
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    /// Default tags applied at creation time. Pairs are `(key, category)`.
    fn default_tags(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Commands this type's owner can use (when actor is THIS object).
    fn default_commands(&self) -> Vec<Arc<dyn Command>> {
        Vec::new()
    }

    /// Lock map — kind → expression. Recognized kinds:
    /// `"view"`, `"examine"`, `"control"`, `"puppet"`, `"use"`.
    fn locks(&self) -> HashMap<&'static str, &'static str> {
        HashMap::new()
    }
}

/// Built-in type classes (Character, Room, Item, Exit, Mob).
pub mod builtins {
    use super::*;

    /// A player-controllable character. Default cmds: look, say, quit, who.
    pub struct CharacterType;

    impl TypeClass for CharacterType {
        fn key(&self) -> &'static str {
            "character"
        }
        fn description(&self) -> &'static str {
            "A player or NPC character that can be puppeted by an account."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("hp".into(), serde_json::json!(10));
            m.insert("desc".into(), serde_json::json!("A nondescript person."));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("character".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "perm(builder)");
            m.insert("puppet", "perm(player)");
            m
        }
    }

    /// A room: a container for other objects. No default commands.
    pub struct RoomType;

    impl TypeClass for RoomType {
        fn key(&self) -> &'static str {
            "room"
        }
        fn description(&self) -> &'static str {
            "A spatial region that contains characters, items, and exits."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("desc".into(), serde_json::json!("An empty space."));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("room".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "perm(builder)");
            m
        }
    }

    /// A pickup-able item.
    pub struct ItemType;

    impl TypeClass for ItemType {
        fn key(&self) -> &'static str {
            "item"
        }
        fn description(&self) -> &'static str {
            "A movable object — pickable, droppable, droppable into containers."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("weight".into(), serde_json::json!(1));
            m.insert("desc".into(), serde_json::json!("An ordinary item."));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("item".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "all()");
            m.insert("use", "all()");
            m
        }
    }

    /// An exit between two rooms. The `destination` attribute holds the target room UID.
    pub struct ExitType;

    impl TypeClass for ExitType {
        fn key(&self) -> &'static str {
            "exit"
        }
        fn description(&self) -> &'static str {
            "A traversable exit; `destination` attribute names the target room."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("destination".into(), serde_json::json!(null));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("exit".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("traverse", "all()");
            m
        }
    }

    /// A non-player NPC. No default cmds; behavior driven by scripts (Plan 5).
    pub struct MobType;

    impl TypeClass for MobType {
        fn key(&self) -> &'static str {
            "mob"
        }
        fn description(&self) -> &'static str {
            "A non-player character driven by scripted AI."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("hp".into(), serde_json::json!(5));
            m.insert("aggressive".into(), serde_json::json!(false));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("mob".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "perm(builder)");
            m
        }
    }
}
