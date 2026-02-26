pub use steel_plugin_macros::{on_disable, on_enable, plugin_meta};

use crate::event::{EventHandlerFlags, EventId};

pub mod event;
pub mod types;

#[must_use]
pub fn pack_handler(event_id: EventId, priority: i8, flags: EventHandlerFlags) -> u32 {
    let event_id = event_id as u32 & 0xFFFF;
    let priority = u32::from(priority as u8) & 0xFF;
    let flags = u32::from(flags.bits()) & 0xFF;

    (event_id << 16) | (priority << 8) | flags
}

#[must_use]
pub const fn unpack_handler(packed: u32) -> (EventId, i8, EventHandlerFlags) {
    let event_id = EventId::from_repr(((packed >> 16) & 0xFFFF) as u16).unwrap();
    let priority = ((packed >> 8) & 0xFF) as u8 as i8;
    let flags = EventHandlerFlags::from_bits_truncate((packed & 0xFF) as u8);

    (event_id, priority, flags)
}

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
