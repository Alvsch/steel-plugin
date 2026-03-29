use steel_plugin_sdk::event::PlayerJoinEvent;
use steel_plugin_sdk::{event_handler, info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "listening",
    description = "A plugin that listens to an event",
    version = "0.1.0",
    depends = [],
);

#[event_handler(priority = -1)]
fn test_handler(mut event: PlayerJoinEvent) -> Option<PlayerJoinEvent> {
    info!("{:?}", event);

    event.username = "Alvsch1".to_string();
    Some(event)
}

#[on_enable]
pub fn on_enable() {
    info!("hello from the listening!");
}

#[on_disable]
pub fn on_disable() {}
