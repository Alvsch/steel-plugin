use steel_plugin_sdk::event::{PlayerJoinEvent, PlayerLeaveEvent};
use steel_plugin_sdk::objects::player::{Name, Position};
use steel_plugin_sdk::{event_handler, info, on_disable, on_enable, plugin_meta};

plugin_meta!();

#[event_handler(priority = -1)]
fn test_handler(event: PlayerJoinEvent) -> Option<PlayerJoinEvent> {
    let (name, position) = event.player.fetch::<(Name, Position)>().unwrap();
    info!("name={name}, position={position}");

    event
        .player
        .batch()
        .send_message(format!("Welcome {name}!"))
        .send();

    None
}

#[event_handler]
fn test_handler(_event: PlayerLeaveEvent) -> Option<PlayerLeaveEvent> {
    info!("goodbye");
    None
}

#[on_enable]
pub fn on_enable() {
    info!("hello from the listening!");
}

#[on_disable]
pub fn on_disable() {}
