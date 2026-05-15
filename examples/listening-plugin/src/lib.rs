use steel_plugin_sdk::{Plugin, info, plugin_meta};

plugin_meta!();

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
