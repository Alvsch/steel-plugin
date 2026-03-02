use serde::{Serialize, de::DeserializeOwned};
use steel_plugin_sdk::utils::fat::FatPtr;
use wasmtime::{AsContext, AsContextMut, Caller, Extern, Memory, TypedFunc};

use crate::PluginHostData;

pub struct PluginMemory<'a, S> {
    memory: Memory,
    store: &'a mut S,
}

impl<'a, S> PluginMemory<'a, S>
where
    S: AsContext<Data = PluginHostData> + AsContextMut<Data = PluginHostData>,
{
    pub const fn new(memory: Memory, store: &'a mut S) -> Self {
        Self { memory, store }
    }

    pub fn read(&self, fat: FatPtr) -> &[u8] {
        &self.memory.data(&self.store)[fat.ptr() as usize..(fat.ptr() + fat.len()) as usize]
    }

    pub fn write(&mut self, ptr: u32, src: &[u8]) {
        self.memory.data_mut(&mut self.store)[ptr as usize..ptr as usize + src.len()]
            .copy_from_slice(src);
    }

    pub fn read_msgpack<T: DeserializeOwned>(&self, fat: FatPtr) -> T {
        let bytes = self.read(fat);
        rmp_serde::from_slice::<T>(bytes).unwrap()
    }

    pub async fn write_msgpack<T: Serialize>(
        &mut self,
        value: &T,
        alloc: &TypedFunc<u32, u32>,
    ) -> FatPtr {
        let bytes = rmp_serde::to_vec(value).unwrap();
        let len = bytes.len() as u32;
        let ptr = alloc.call_async(&mut self.store, len).await.unwrap();
        self.write(ptr, &bytes);
        FatPtr::new(ptr, len).expect("alloc returned a null fat pointer")
    }
}

impl<'a, 'b> From<&'a mut Caller<'b, PluginHostData>>
    for PluginMemory<'a, Caller<'b, PluginHostData>>
{
    fn from(caller: &'a mut Caller<'b, PluginHostData>) -> Self {
        let memory = caller
            .get_export("memory")
            .and_then(Extern::into_memory)
            .unwrap();
        Self {
            memory,
            store: caller,
        }
    }
}
