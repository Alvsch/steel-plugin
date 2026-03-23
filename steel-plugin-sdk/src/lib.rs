pub use steel_plugin_macros::{event_handler, on_disable, on_enable, plugin_meta, rpc_export};

pub mod event;
pub mod rpc;
pub mod types;
pub mod utils;

pub(crate) mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub unsafe fn info(ptr: u32, len: u32);
        // rpc
        pub unsafe fn rpc_register(export_name: u64, fn_table_index: u32);
        pub unsafe fn rpc_resolve_plugin(name: u64) -> u32;
        pub unsafe fn rpc_resolve_method(plugin_id: u32, name: u64) -> u32;
        pub unsafe fn rpc_dispatch(plugin_id: u32, method_id: u32, data: u64) -> u64;
        // event
        pub unsafe fn event_subscribe(topic_id: u32, fn_table_index: u32, priority: i32);
    }
}

#[doc(hidden)]
pub mod __export {
    pub use crate::host::info;
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::info(message.as_ptr() as u32, message.len() as u32);
        }
    };
}
