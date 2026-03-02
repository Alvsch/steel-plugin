use crate::EventRegistry;
use steel_plugin_sdk::event::{
    BlockBreakEvent, BlockPlaceEvent, Event, PlayerChatEvent, PlayerJoinEvent, PlayerLeaveEvent,
};
use wasmparser::{Parser, Payload};

pub mod memory;
mod sorting;

pub use sorting::sort_plugins;

pub fn read_custom_section<'a>(
    bytes: &'a [u8],
    name: &str,
) -> wasmparser::Result<Option<&'a [u8]>> {
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

pub fn register_default_events(registry: &EventRegistry) {
    registry.register_event(PlayerJoinEvent::NAME.to_string());
    registry.register_event(PlayerLeaveEvent::NAME.to_string());
    registry.register_event(PlayerChatEvent::NAME.to_string());
    registry.register_event(BlockBreakEvent::NAME.to_string());
    registry.register_event(BlockPlaceEvent::NAME.to_string());
}
