use std::mem::forget;
use std::slice;
use steel_plugin_sdk::rpc::rpc_register;
use steel_plugin_sdk::{export, info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "provider",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[export]
pub fn get_balance(data: Vec<u8>) -> Vec<u8> {
    let msg = str::from_utf8(&data).unwrap();
    info(&format!("get_balance: {msg}"));

    Vec::new()
}

#[on_enable]
pub fn on_enable() {
    info("hello from the provider!");
    rpc_register("get_balance");
}

#[on_disable]
pub fn on_disable() {}
