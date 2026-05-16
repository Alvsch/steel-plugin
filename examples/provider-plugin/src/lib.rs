use steel_plugin_sdk::{Plugin, info, rpc_export};

pub struct ProviderPlugin;

impl Plugin for ProviderPlugin {
    fn on_enable() {
        info!("hello from the provider!");
    }

    fn on_disable() {
        info!("provider disabled");
    }
}

#[rpc_export]
fn greet(data: &[u8]) -> Option<Vec<u8>> {
    info!("hello {}", String::from_utf8_lossy(data));
    None
}

steel_plugin_sdk::plugin_export!(ProviderPlugin);
