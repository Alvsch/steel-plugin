use std::{fs, slice};

use steel_plugin_sdk::{init, plugin_meta, print};

init!();

plugin_meta!(
    name = "steel-plugin",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[unsafe(no_mangle)]
pub extern "C" fn on_load(ptr: u32, len: u32) {
    let name = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    let name = str::from_utf8(name).unwrap();

    fs::write("/data/latest.log", "hello").unwrap();
    print(&format!("Hello, {name}!"));
}

#[unsafe(no_mangle)]
pub extern "C" fn on_unload() {}
