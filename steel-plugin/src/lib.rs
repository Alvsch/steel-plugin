use std::fs;

use serde::{Deserialize, Serialize};
use steel_plugin_sdk::event::Event;
use steel_plugin_sdk::{
    event::{PlayerJoinEvent, result::EventResult},
    event_handler, info, on_disable, on_enable, plugin_meta, register_event, register_handler,
};
use uuid::Uuid;

plugin_meta!(
    name = "steel-plugin",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[derive(Debug, Serialize, Deserialize)]
pub struct MyEvent {
    name: String,
}

impl Event for MyEvent {
    const NAME: &str = "MyEvent";

    fn cancelled(&self) -> bool {
        false
    }
}

#[event_handler]
pub fn handle_join(mut event: PlayerJoinEvent) -> EventResult<PlayerJoinEvent> {
    info(&format!("{} joined the game!", event.player));
    event.player = Uuid::new_v4();

    EventResult::modified(&event)
}

#[on_enable]
pub fn on_enable() {
    register_event::<MyEvent>();
    register_handler!(handle_join);

    fs::write("/latest.log", "hello").unwrap();
    info("Hello, World!");
}

#[on_disable]
pub fn on_disable() {}
