pub use steel_plugin_macros::{event_handler, on_disable, on_enable, plugin_meta, rpc_export};

pub mod event;
pub mod export;
pub mod objects;
pub mod rpc;
pub mod utils;

pub use steel_plugin_core::STEEL_API_VERSION;

pub(crate) mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub unsafe fn error(ptr: u32, len: u32);
        pub unsafe fn warn(ptr: u32, len: u32);
        pub unsafe fn info(ptr: u32, len: u32);
        pub unsafe fn debug(ptr: u32, len: u32);
        pub unsafe fn trace(ptr: u32, len: u32);
        // rpc
        pub unsafe fn rpc_resolve_plugin(name: u64) -> u32;
        pub unsafe fn rpc_resolve_method(plugin_id: u32, name: u64) -> u32;
        pub unsafe fn rpc_dispatch(plugin_id: u32, method_id: u32, data: u64) -> u64;
        // objects
        pub unsafe fn object_fetch(entity_key: u64, queries_ptr: u32, queries_len: u32) -> u64;
        pub unsafe fn object_batch_dispatch(entity_key: u64, ptr: u32, len: u32);
    }
}

#[doc(hidden)]
pub mod __export {
    pub use crate::host::debug;
    pub use crate::host::error;
    pub use crate::host::info;
    pub use crate::host::trace;
    pub use crate::host::warn;
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::error(message.as_ptr() as u32, message.len() as u32);
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::warn(message.as_ptr() as u32, message.len() as u32);
        }
    };
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

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::debug(message.as_ptr() as u32, message.len() as u32);
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        unsafe {
            $crate::__export::trace(message.as_ptr() as u32, message.len() as u32);
        }
    };
}
