use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf};
use steel_plugin_sdk::event::handler::EventHandler;
use steel_plugin_sdk::utils::fat::FatPtr;
use tracing::info;
use wasmparser::{Parser, Payload};
use wasmtime::{Caller, Linker};
use wasmtime_wasi::p1::wasi_snapshot_preview1::add_to_linker;

pub use wasmtime;

pub use crate::event_registry::EventRegistry;
pub use crate::exports::PluginExports;
pub use crate::loader::{PluginHostData, PluginLoader, PluginLoaderError};
pub use crate::manager::PluginManager;
use crate::utils::memory::PluginMemory;
pub use instance::PluginInstance;
use steel_plugin_sdk::event::{
    BlockBreakEvent, BlockPlaceEvent, Event, PlayerChatEvent, PlayerJoinEvent, PlayerLeaveEvent,
};

mod event_registry;
mod exports;
mod instance;
mod loader;
mod manager;
mod utils;

fn read_custom_section<'a>(bytes: &'a [u8], name: &str) -> wasmparser::Result<Option<&'a [u8]>> {
    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::CustomSection(reader) if reader.name() == name => {
                return Ok(Some(reader.data()));
            }
            _ => {}
        }
    }
    Ok(None)
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub depends: Box<[String]>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

pub fn configure_linker(linker: &mut Linker<PluginHostData>) {
    add_to_linker(linker, |data: &mut PluginHostData| &mut data.wasi).unwrap();
    linker
        .func_wrap(
            "host",
            "info",
            |mut caller: Caller<PluginHostData>, ptr: u32, len: u32| {
                let memory = PluginMemory::from(&mut caller);
                let message: String = memory.read_msgpack(FatPtr::new(ptr, len).unwrap());

                let plugin_name = caller.data().name.as_str();
                info!("[{plugin_name}] {message}");
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "register_handler",
            |mut caller: Caller<PluginHostData>, ptr: u32, len: u32| {
                let memory = PluginMemory::from(&mut caller);
                let handler: EventHandler = memory.read_msgpack(FatPtr::new(ptr, len).unwrap());

                let registry = &*caller.data().registry;
                let plugin_name = caller.data().name.clone();
                registry.register_handler(plugin_name, handler);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "register_event",
            |mut caller: Caller<PluginHostData>, ptr: u32, len: u32| {
                let memory = PluginMemory::from(&mut caller);
                let event_name: String = memory.read_msgpack(FatPtr::new(ptr, len).unwrap());

                let registry = &*caller.data().registry;
                registry.register_event(event_name);
            },
        )
        .unwrap();
}

pub fn register_default_events(registry: &EventRegistry) {
    registry.register_event(PlayerJoinEvent::NAME.to_string());
    registry.register_event(PlayerLeaveEvent::NAME.to_string());
    registry.register_event(PlayerChatEvent::NAME.to_string());
    registry.register_event(BlockBreakEvent::NAME.to_string());
    registry.register_event(BlockPlaceEvent::NAME.to_string());
}
