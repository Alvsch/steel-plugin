pub use steel_plugin_macros::{export, on_disable, on_enable, plugin_meta};

pub mod rpc;
pub mod types;
pub mod utils;

pub(crate) mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub unsafe fn info(ptr: u32, len: u32);
        // rpc
        pub unsafe fn rpc_register(export_name: u64);
        pub unsafe fn rpc_resolve_plugin(name: u64) -> u32;
        pub unsafe fn rpc_resolve_method(plugin_id: u32, name: u64) -> u32;
        pub unsafe fn rpc_dispatch(plugin_id: u32, method_id: u32, data: u64) -> u64;
    }
}

pub fn info(message: &str) {
    unsafe {
        host::info(message.as_ptr() as u32, message.len() as u32);
    }
}
