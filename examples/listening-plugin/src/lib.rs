use steel_plugin_sdk::{Plugin, event::PlayerJoinEvent, event_handler, info};

#[event_handler]
fn event_handler(event: &mut PlayerJoinEvent) {}

pub struct ListeningPlugin;

impl Plugin for ListeningPlugin {
    fn on_enable() {
        info!("hello from the listening!");
    }

    fn on_disable() {
        info!("goodbye from the listening!");
    }
}

steel_plugin_sdk::plugin_export!(ListeningPlugin);
