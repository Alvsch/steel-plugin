use wasmtime::{Instance, Store, TypedFunc};

use crate::PluginHostData;

pub struct PluginExports {
    pub alloc: TypedFunc<u32, u32>,
    pub dealloc: TypedFunc<(u32, u32), ()>,
    pub on_enable: TypedFunc<(), ()>,
    pub on_disable: TypedFunc<(), ()>,
}

impl PluginExports {
    pub fn resolve(
        instance: &Instance,
        store: &mut Store<PluginHostData>,
    ) -> wasmtime::Result<Self> {
        Ok(Self {
            alloc: instance.get_typed_func(&mut *store, "alloc")?,
            dealloc: instance.get_typed_func(&mut *store, "dealloc")?,
            on_enable: instance.get_typed_func(&mut *store, "on_enable")?,
            on_disable: instance.get_typed_func(&mut *store, "on_disable")?,
        })
    }
}
