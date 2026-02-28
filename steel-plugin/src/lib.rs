use std::fs;

use steel_plugin_sdk::{
    event::{
        Event, EventHandlerFlags, PlayerJoinEvent, handler::EventHandler, result::EventResult,
    },
    info, on_disable, on_enable, on_event, plugin_meta,
};
use uuid::Uuid;

plugin_meta!(
    name = "steel-plugin",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[on_event]
pub fn on_event(mut event: PlayerJoinEvent) -> EventResult {
    info(&format!("{} joined the game!", event.player));
    event.player = Uuid::new_v4();

    let data = rmp_serde::to_vec(&event).unwrap();
    EventResult::modified(data)
}

#[on_enable]
pub fn on_enable() {
    steel_plugin_sdk::register_handler(&EventHandler {
        event_name: PlayerJoinEvent::NAME.into(),
        priority: 0,
        flags: EventHandlerFlags::empty(),
    });
    fs::write("/latest.log", "hello").unwrap();
    info("Hello, World!");
}

#[on_disable]
pub fn on_disable() {}
