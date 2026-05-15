use steel_plugin_sdk::{Guest, info, plugin_meta};

plugin_meta!();

pub struct ProviderPlugin;

impl Guest for ProviderPlugin {
    fn on_enable() {
        info!("hello from the provider!");
    }

    fn on_disable() {
        info!("provider disabled");
    }

    fn on_load() -> Vec<u8> {
        let slice = ::steel_plugin_sdk::export::iter::<::steel_plugin_sdk::export::Exported>()
            .cloned()
            .map(::steel_plugin_sdk::export::ExportedId::from)
            .collect::<Vec<_>>();
        ::rmp_serde::to_vec(&slice).unwrap()
    }

    fn rpc(_method_id: u32, _data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }

    fn event_handler(_handler_id: u32, _data: Vec<u8>) {}
}

steel_plugin_sdk::plugin_export!(ProviderPlugin);
