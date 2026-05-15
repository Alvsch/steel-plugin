use steel_plugin_sdk::{Plugin, info};

pub struct ProviderPlugin;

impl Plugin for ProviderPlugin {
    fn on_enable() {
        info!("hello from the provider!");
    }

    fn on_disable() {
        info!("provider disabled");
    }
}

steel_plugin_sdk::plugin_export!(ProviderPlugin);
