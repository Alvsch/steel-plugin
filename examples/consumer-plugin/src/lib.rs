use steel_plugin_sdk::rpc::{rpc_dispatch, rpc_resolve_method, rpc_resolve_plugin};
use steel_plugin_sdk::{info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "consumer",
    version = "0.1.0",
    api_version = 1,
    depends = ["provider"],
);

#[on_enable]
pub fn on_enable() {
    info!("hello from the consumer!");

    let plugin_id = rpc_resolve_plugin("provider");
    let method_id = rpc_resolve_method(plugin_id, "get_balance");
    let result =
        rpc_dispatch(plugin_id, method_id, b"hello").and_then(|x| String::from_utf8(x).ok());

    info!("{result:?}");
}

#[on_disable]
pub fn on_disable() {}
