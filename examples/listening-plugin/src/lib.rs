use steel_plugin_sdk::event::{PlayerJoinEvent, event_subscribe};
use steel_plugin_sdk::{event_handler, info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "listening",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[event_handler]
fn test_handler(event: PlayerJoinEvent) {
    info!("{:?}", event);
}

#[on_enable]
pub fn on_enable() {
    info!("hello from the listening!");

    event_subscribe(0, test_handler, 0);
}

#[on_disable]
pub fn on_disable() {}
