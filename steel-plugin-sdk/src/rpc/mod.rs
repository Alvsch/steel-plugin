// Plugin A -> Host -> Plugin B
//
// plugin defined interface
// accessible to host for registration and plugin for usage
//
// rpc_plugin_id(name: &str) -> u32
//
// register interface
// rpc_register(export_name: &str);
//
// call interface
// rpc_resolve(plugin_id: u32, name: &str) -> u32
// rpc_dispatch(plugin_id: u32, method_id: u32, data: &[u8]);
//
// pub unsafe fn rpc_register(export_name: u64);
// pub unsafe fn rpc_plugin_id(plugin_name: u64) -> u32;
// pub unsafe fn rpc_resolve(plugin_id: u32, name: u64) -> u32;
// pub unsafe fn rpc_dispatch(plugin_id: u32, method_id: u32, data: u64);

use crate::host;
use crate::utils::fat::FatPtr;

pub fn rpc_register(export_name: &str) {
    let fat = FatPtr::new(export_name.as_ptr() as u32, export_name.len() as u32).unwrap();
    unsafe {
        host::rpc_register(fat.pack());
    }
}

#[must_use]
pub fn rpc_plugin_id(plugin_name: &str) -> u32 {
    let fat = FatPtr::new(plugin_name.as_ptr() as u32, plugin_name.len() as u32).unwrap();
    unsafe { host::rpc_plugin_id(fat.pack()) }
}

#[must_use]
pub fn rpc_resolve(plugin_id: u32, name: &str) -> u32 {
    let fat = FatPtr::new(name.as_ptr() as u32, name.len() as u32).unwrap();
    unsafe { host::rpc_resolve(plugin_id, fat.pack()) }
}

pub fn rpc_dispatch(plugin_id: u32, method_id: u32, data: &[u8]) {
    let fat = FatPtr::new(data.as_ptr() as u32, data.len() as u32).unwrap();
    unsafe {
        host::rpc_dispatch(plugin_id, method_id, fat.pack());
    }
}
