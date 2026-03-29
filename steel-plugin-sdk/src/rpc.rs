use std::num::NonZeroU32;

use crate::host;
use crate::utils::fat::FatPtr;

pub type PluginId = NonZeroU32;
pub type MethodId = NonZeroU32;

#[must_use]
pub fn rpc_resolve_plugin(plugin_name: &str) -> u32 {
    let fat = FatPtr::new(plugin_name.as_ptr() as u32, plugin_name.len() as u32)
        .expect("ptr is never null");
    unsafe { host::rpc_resolve_plugin(fat.pack()) }
}

#[must_use]
pub fn rpc_resolve_method(plugin_id: u32, name: &str) -> u32 {
    let fat = FatPtr::new(name.as_ptr() as u32, name.len() as u32).expect("ptr is never null");
    unsafe { host::rpc_resolve_method(plugin_id, fat.pack()) }
}

#[allow(clippy::must_use_candidate)]
pub fn rpc_dispatch(plugin_id: u32, method_id: u32, data: &[u8]) -> Option<Vec<u8>> {
    let fat_data = FatPtr::new(data.as_ptr() as u32, data.len() as u32).expect("ptr is never null");
    let result = unsafe { host::rpc_dispatch(plugin_id, method_id, fat_data.pack()) };
    let fat_result = FatPtr::unpack(result)?;
    Some(unsafe {
        Vec::from_raw_parts(
            fat_result.ptr() as *mut u8,
            fat_result.len() as usize,
            fat_result.len() as usize,
        )
    })
}
