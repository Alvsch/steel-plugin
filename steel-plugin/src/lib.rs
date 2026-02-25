use std::fs;

use steel_plugin_sdk::{info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "steel-plugin",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[on_enable]
pub fn on_enable() {
    fs::write("/latest.log", "hello").unwrap();
    info("Hello, World!");
}

#[on_disable]
pub fn on_disable() {}
