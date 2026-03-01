use std::fs;

use steel_plugin_sdk::{
    event::{PlayerJoinEvent, result::EventResult},
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
pub fn handle_join(mut event: PlayerJoinEvent) -> EventResult {
    info(&format!("{} joined the game!", event.player));
    event.player = Uuid::new_v4();

    let data = rmp_serde::to_vec(&event).unwrap();
    EventResult::modified(data)
}

#[on_enable]
pub fn on_enable() {
    steel_plugin_sdk::register_event!(handle_join, PlayerJoinEvent);
    fs::write("/latest.log", "hello").unwrap();
    info("Hello, World!");
}

#[on_disable]
pub fn on_disable() {}
