pub use steel_plugin_macros::{on_disable, on_enable, on_event, plugin_meta};

use crate::{
    event::{EventHandlerFlags, EventId},
    utils::pack_handler,
};

pub mod event;
pub mod types;
pub mod utils;

mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub unsafe fn info(ptr: u32, len: u32);
        pub unsafe fn register_handler(packed: u32);
    }
}

pub fn register_handler(event_id: EventId, priority: i8, flags: EventHandlerFlags) {
    let packed = pack_handler(event_id, priority, flags);
    unsafe {
        host::register_handler(packed);
    }
}

pub fn info(message: &str) {
    unsafe {
        host::info(message.as_ptr() as u32, message.len() as u32);
    }
}
