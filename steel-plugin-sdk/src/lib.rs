pub use steel_plugin_macros::plugin_meta;

mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub unsafe fn schedule_timer(delay_ms: u64, repeat_ms: u64, callback_id: u32);
        pub unsafe fn info(ptr: u32, len: u32);
    }
}

pub fn schedule_timer(delay_ms: u64, repeat_ms: u64, callback_id: u32) {
    unsafe {
        host::schedule_timer(delay_ms, repeat_ms, callback_id);
    }
}

pub fn info(message: &str) {
    unsafe {
        host::info(message.as_ptr() as u32, message.len() as u32);
    }
}
