use steel_plugin_sdk::event::event_subscribe;
use steel_plugin_sdk::utils::fat::FatPtr;
use steel_plugin_sdk::{info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "listening",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[on_enable]
pub fn on_enable() {
    info("hello from the listening!");

    event_subscribe(
        0,
        |packed| {
            let fat_ptr = FatPtr::unpack(packed).unwrap();
            info(&format!("got a ptr: {fat_ptr:?}"));
        },
        0,
    );
}

#[on_disable]
pub fn on_disable() {}
