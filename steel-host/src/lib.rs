use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf};

pub use wasmtime;

pub use crate::event_registry::EventRegistry;
pub use crate::exports::PluginExports;
pub use crate::loader::{PluginHostData, PluginLoader, PluginLoaderError};
pub use crate::manager::PluginManager;
pub use instance::PluginInstance;
use steel_plugin_sdk::event::{
    BlockBreakEvent, BlockPlaceEvent, Event, PlayerChatEvent, PlayerJoinEvent, PlayerLeaveEvent,
};

mod event_registry;
mod exports;
mod instance;
pub mod linker;
mod loader;
mod manager;
pub mod rpc;
mod utils;

#[derive(Debug, Deserialize, Clone)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub depends: Box<[String]>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

pub fn register_default_events(registry: &EventRegistry) {
    registry.register_event(PlayerJoinEvent::NAME.to_string());
    registry.register_event(PlayerLeaveEvent::NAME.to_string());
    registry.register_event(PlayerChatEvent::NAME.to_string());
    registry.register_event(BlockBreakEvent::NAME.to_string());
    registry.register_event(BlockPlaceEvent::NAME.to_string());
}
