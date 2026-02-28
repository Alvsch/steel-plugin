use std::fs;

use steel_plugin_sdk::{
    event::{EventHandlerFlags, EventId, PluginEvent, result::EventResult},
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
pub fn on_event(event: &[u8]) -> EventResult {
    let event: PluginEvent = rmp_serde::from_slice(event).unwrap();
    match event {
        PluginEvent::PlayerJoinEvent(mut event) => {
            info(&format!("{} joined the game!", event.player));
            event.player = Uuid::new_v4();

            let data = rmp_serde::to_vec(&PluginEvent::PlayerJoinEvent(event)).unwrap();
            return EventResult::modified(data);
        }
        _ => (),
    }

    EventResult::default()
}

#[on_enable]
pub fn on_enable() {
    steel_plugin_sdk::register_handler(EventId::PlayerJoinEvent, 0, EventHandlerFlags::empty());
    fs::write("/latest.log", "hello").unwrap();
    info("Hello, World!");
}

#[on_disable]
pub fn on_disable() {}
