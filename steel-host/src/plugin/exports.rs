use wasmtime::{Instance, Store, TypedFunc};

use crate::PluginState;

pub type AllocFunc = TypedFunc<u32, u32>;
pub type DeallocFunc = TypedFunc<(u32, u32), ()>;

#[derive(Clone)]
pub struct PluginExports {
    /// (`ptr`, `len`)
    pub alloc: AllocFunc,
    /// (`ptr`, `len`)
    pub dealloc: DeallocFunc,
    pub on_enable: TypedFunc<(), ()>,
    pub on_disable: TypedFunc<(), ()>,
}

impl PluginExports {
    pub fn resolve(instance: &Instance, store: &mut Store<PluginState>) -> wasmtime::Result<Self> {
        Ok(Self {
            alloc: instance.get_typed_func(&mut *store, "alloc")?,
            dealloc: instance.get_typed_func(&mut *store, "dealloc")?,
            on_enable: instance.get_typed_func(&mut *store, "on_enable")?,
            on_disable: instance.get_typed_func(&mut *store, "on_disable")?,
        })
    }
}
