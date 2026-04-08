use steel_plugin_sdk::event::PlayerJoinEvent;
use steel_plugin_sdk::objects::player::{Health, Name};
use steel_plugin_sdk::{event_handler, info, on_disable, on_enable, plugin_meta};

plugin_meta!();

#[event_handler(priority = -1)]
fn test_handler(event: PlayerJoinEvent) -> Option<PlayerJoinEvent> {
    info!("{:?}", event);

    let (name, health) = event.player.fetch::<(Name, Health)>();
    info!("before: name={name}, health={health}");

    event
        .player
        .batch()
        .send_message(format!("Welcome {name}!"))
        .set_health(17.0)
        .send();

    let (_, new_health) = event.player.fetch::<(Name, Health)>();
    info!("after: health={new_health}");

    Some(event)
}

#[on_enable]
pub fn on_enable() {
    info!("hello from the listening!");
}

#[on_disable]
pub fn on_disable() {}
