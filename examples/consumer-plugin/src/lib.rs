use steel_plugin_sdk::{Plugin, info, rpc};

pub struct ConsumerPlugin;

impl Plugin for ConsumerPlugin {
    fn on_enable() {
        info!("hello from the consumer!");
        let plugin_id = rpc::resolve_plugin("provider-plugin").unwrap();
        rpc::dispatch(plugin_id, "greet", b"Steve");
    }

    fn on_disable() {
        info!("consumer disabled");
    }
}

steel_plugin_sdk::plugin_export!(ConsumerPlugin, {
    depends: ["provider-plugin"]
});
