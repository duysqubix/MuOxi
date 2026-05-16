//! Central registry for types, commands, and hooks.

use crate::hooks::{Hook, Hooks};
use crate::prelude::Command;
use crate::scheduler::ScriptHandler;
use crate::typeclass::TypeClass;
use crate::world::WorldApi;
use dashmap::DashMap;
use std::sync::Arc;

/// All extension points a downstream developer registers against.
pub struct Registry {
    types: DashMap<&'static str, Arc<dyn TypeClass>>,
    commands: DashMap<String, Arc<dyn Command>>,
    script_handlers: DashMap<String, Arc<dyn ScriptHandler>>,
    /// hook collection — exposed so callers can fire events directly
    pub hooks: Hooks,
    /// shared world facade
    pub world: Arc<WorldApi>,
}

impl Registry {
    /// Empty registry, no built-ins.
    pub fn new(world: Arc<WorldApi>) -> Self {
        Self {
            types: DashMap::new(),
            commands: DashMap::new(),
            script_handlers: DashMap::new(),
            hooks: Hooks::new(),
            world,
        }
    }

    /// Register a `TypeClass`. Replaces any existing registration with the same key.
    pub fn register_type(&self, t: Arc<dyn TypeClass>) {
        self.types.insert(t.key(), t);
    }

    /// Register a `Command` by its primary name and aliases. The same `Arc`
    /// is stored under each key so registry lookups by alias work.
    pub fn register_command(&self, c: Arc<dyn Command>) {
        let name = c.name().to_string();
        self.commands.insert(name, c.clone());
        for alias in c.aliases() {
            self.commands.insert(alias.to_string(), c.clone());
        }
    }

    /// Register a `Hook`. Hooks fire in registration order.
    pub fn register_hook(&self, h: Arc<dyn Hook>) {
        self.hooks.register(h);
    }

    /// Look up a type class by key.
    pub fn get_type(&self, key: &str) -> Option<Arc<dyn TypeClass>> {
        self.types.get(key).map(|r| r.clone())
    }

    /// Look up a command by name or alias (case-insensitive).
    pub fn resolve_command(&self, input: &str) -> Option<Arc<dyn Command>> {
        let token = input.split_whitespace().next()?.to_lowercase();
        self.commands.get(&token).map(|r| r.clone())
    }

    /// Register a script handler. Replaces any previous handler with the same key.
    pub fn register_script_handler(&self, h: Arc<dyn ScriptHandler>) {
        self.script_handlers.insert(h.key().to_string(), h);
    }

    /// Resolve a script handler by key.
    pub fn script_handler(&self, key: &str) -> Option<Arc<dyn ScriptHandler>> {
        self.script_handlers.get(key).map(|r| r.clone())
    }

    /// Bulk-register the framework's built-in type classes.
    pub fn register_builtin_types(&self) {
        use crate::typeclass::builtins::*;
        self.register_type(Arc::new(CharacterType));
        self.register_type(Arc::new(RoomType));
        self.register_type(Arc::new(ItemType));
        self.register_type(Arc::new(ExitType));
        self.register_type(Arc::new(MobType));
    }
}
