pub use steel_plugin_macros::{event_handler, on_disable, on_enable, plugin_meta, register_event};

use crate::event::handler::EventHandler;

pub mod event;
pub mod types;

mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub unsafe fn info(ptr: u32, len: u32);
        pub unsafe fn register_handler(ptr: u32, len: u32);
    }
}

pub fn register_handler(handler: &EventHandler) {
    let data = rmp_serde::to_vec(handler).unwrap();
    unsafe {
        host::register_handler(data.as_ptr() as u32, data.len() as u32);
    }
}

pub fn info(message: &str) {
    unsafe {
        host::info(message.as_ptr() as u32, message.len() as u32);
    }
}
