use std::fs;

use steel_plugin_sdk::{
    event::{EventHandlerFlags, EventId, EventResult, PlayerJoinEvent, PlayerLeaveEvent},
    info, on_disable, on_enable, on_event, plugin_meta,
};

plugin_meta!(
    name = "steel-plugin",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[on_event]
pub fn on_event(event_id: EventId, event: &[u8]) -> EventResult {
    match EventId::from_repr(event_id as u16).unwrap() {
        EventId::PlayerJoinEvent => {
            let event: PlayerJoinEvent = rmp_serde::from_slice(event).unwrap();
            info(&format!("{:#?}", event));
        }
        EventId::PlayerLeaveEvent => {
            let event: PlayerLeaveEvent = rmp_serde::from_slice(event).unwrap();
            info(&format!("{:#?}", event));
        }
        _ => (),
    };
    EventResult::empty()
}

#[on_enable]
pub fn on_enable() {
    steel_plugin_sdk::register_handler(EventId::PlayerJoinEvent, 0, EventHandlerFlags::empty());
    fs::write("/latest.log", "hello").unwrap();
    info("Hello, World!");
}

#[on_disable]
pub fn on_disable() {}
