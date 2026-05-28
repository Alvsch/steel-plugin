use steel_plugin_sdk::{Plugin, info, rpc};

pub struct ConsumerPlugin;

impl Plugin for ConsumerPlugin {
    fn on_enable() {
        info!("hello from the consumer!");
        let method =
            rpc::resolve_method("provider-plugin", "greet").expect("failed to resolve method");
        method.dispatch(b"Steve");
    }

    fn on_disable() {
        info!("consumer disabled");
    }
}

steel_plugin_sdk::plugin_export!(ConsumerPlugin, {
    depends: ["provider-plugin"]
});
