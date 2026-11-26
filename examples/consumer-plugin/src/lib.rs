use steel_plugin_sdk::{Plugin, info, rpc};

pub struct ConsumerPlugin;

impl Plugin for ConsumerPlugin {
    fn on_enable() {
        info!("hello from the consumer!");
        let response = rpc::call("provider-plugin", "greet", b"Steve")
            .expect("provider greet call should succeed");
        assert!(response.is_none(), "greet is a no-response RPC");

        let error = rpc::call("provider-plugin", "missing", b"")
            .expect_err("unknown provider method should return an error");
        assert_eq!(error, rpc::CallError::UnknownMethod);
    }

    fn on_disable() {
        info!("consumer disabled");
    }
}

steel_plugin_sdk::plugin_export!(ConsumerPlugin, {
    depends: ["provider-plugin"]
});
