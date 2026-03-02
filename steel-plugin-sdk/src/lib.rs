use crate::event::Event;
use crate::event::handler::EventHandler;
pub use steel_plugin_macros::{
    event_handler, export, on_disable, on_enable, plugin_meta, register_handler,
};

pub mod event;
pub mod rpc;
pub mod types;
pub mod utils;

pub(crate) mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub unsafe fn info(ptr: u32, len: u32);
        pub unsafe fn register_handler(ptr: u32, len: u32);
        pub unsafe fn register_event(ptr: u32, len: u32);

        // rpc
        pub unsafe fn rpc_register(export_name: u64);
        pub unsafe fn rpc_plugin_id(name: u64) -> u32;
        pub unsafe fn rpc_resolve(plugin_id: u32, name: u64) -> u32;
        pub unsafe fn rpc_dispatch(plugin_id: u32, method_id: u32, data: u64);
    }
}

pub fn info(message: &str) {
    unsafe {
        host::info(message.as_ptr() as u32, message.len() as u32);
    }
}

pub fn register_handler(handler: &EventHandler) {
    let data = rmp_serde::to_vec(handler).unwrap();
    unsafe {
        host::register_handler(data.as_ptr() as u32, data.len() as u32);
    }
}

pub fn register_event<T: Event>() {
    let data = String::from(T::NAME);
    unsafe {
        host::register_event(data.as_ptr() as u32, data.len() as u32);
    }
}
