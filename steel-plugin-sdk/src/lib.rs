pub use steel_plugin_macros::plugin_meta;

#[macro_export]
macro_rules! init {
    () => {
        #[unsafe(no_mangle)]
        pub extern "C" fn alloc(len: u32) -> u32 {
            let layout = std::alloc::Layout::from_size_align(len as usize, 1).unwrap();
            unsafe { std::alloc::alloc(layout) as u32 }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn dealloc(ptr: u32, len: u32) {
            let layout = std::alloc::Layout::from_size_align(len as usize, 1).unwrap();
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }
    };
}

mod host {
    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        pub fn print(ptr: u32, len: u32);
    }
}

pub fn print(message: &str) {
    unsafe {
        host::print(message.as_ptr() as u32, message.len() as u32);
    }
}
