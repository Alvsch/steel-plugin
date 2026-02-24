use std::fs;

use steel_plugin_sdk::{info, plugin_meta};

plugin_meta!(
    name = "steel-plugin",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[unsafe(no_mangle)]
pub extern "C" fn on_load() {
    fs::write("/latest.log", "hello").unwrap();
    info("Hello, World!");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_unload() {}
