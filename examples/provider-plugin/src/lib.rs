use std::mem::forget;
use std::slice;
use steel_plugin_sdk::rpc::rpc_register;
use steel_plugin_sdk::utils::fat::FatPtr;
use steel_plugin_sdk::{info, on_disable, on_enable, plugin_meta};

plugin_meta!(
    name = "provider",
    version = "0.1.0",
    api_version = 1,
    depends = [],
);

#[unsafe(no_mangle)]
pub extern "C" fn get_balance(fat: u64) -> u64 {
    let fat = FatPtr::unpack(fat).unwrap();
    let data = unsafe { slice::from_raw_parts(fat.ptr() as *const u8, fat.len() as usize) };
    let msg = str::from_utf8(data).unwrap();
    info(&format!("get_balance: {msg}"));

    let data = rmp_serde::to_vec(&1).unwrap();
    let fat = FatPtr::new(data.as_ptr() as u32, data.len() as u32).unwrap();
    forget(data);
    fat.pack()
}

#[on_enable]
pub fn on_enable() {
    info("hello from the provider!");
    rpc_register("get_balance");
}

#[on_disable]
pub fn on_disable() {}
