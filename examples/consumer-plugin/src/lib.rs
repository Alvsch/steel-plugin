use steel_plugin_sdk::{Plugin, info};

pub struct ConsumerPlugin;

impl Plugin for ConsumerPlugin {
    fn on_enable() {
        info!("hello from the consumer!");
    }

    fn on_disable() {
        info!("consumer disabled");
    }
}

steel_plugin_sdk::plugin_export!(ConsumerPlugin, {
    depends: ["provider-plugin"]
});
