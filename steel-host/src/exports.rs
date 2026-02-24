use wasmtime::{Instance, Store, TypedFunc};

use crate::PluginHostData;

pub struct PluginExports {
    pub alloc: TypedFunc<u32, u32>,
    pub dealloc: TypedFunc<(u32, u32), ()>,
    pub on_load: TypedFunc<(), ()>,
    pub on_unload: TypedFunc<(), ()>,
}

impl PluginExports {
    pub fn resolve(instance: &Instance, store: &mut Store<PluginHostData>) -> anyhow::Result<Self> {
        Ok(Self {
            alloc: instance.get_typed_func(&mut *store, "alloc")?,
            dealloc: instance.get_typed_func(&mut *store, "dealloc")?,
            on_load: instance.get_typed_func(&mut *store, "on_load")?,
            on_unload: instance.get_typed_func(&mut *store, "on_unload")?,
        })
    }
}
