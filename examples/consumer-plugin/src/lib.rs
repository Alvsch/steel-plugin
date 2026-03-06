use steel_plugin_sdk::rpc::rpc_dispatch;
use steel_plugin_sdk::{info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "consumer",
    version = "0.1.0",
    api_version = 1,
    depends = ["provider"],
);

#[on_enable]
pub fn on_enable() {
    info("hello from the consumer!");
    rpc_dispatch(0, 1, b"hello");
}

#[on_disable]
pub fn on_disable() {}
