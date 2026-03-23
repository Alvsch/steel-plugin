use steel_plugin_sdk::utils::fat::FatPtr;
use wasmtime::{AsContext, AsContextMut, Memory};

use crate::PluginState;

pub struct PluginMemory<'a, S> {
    store: &'a mut S,
    memory: &'a Memory,
}

impl<'a, S> PluginMemory<'a, S>
where
    S: AsContext<Data = PluginState> + AsContextMut<Data = PluginState>,
{
    #[inline]
    pub const fn new(store: &'a mut S, memory: &'a Memory) -> Self {
        Self { store, memory }
    }

    pub fn read(&self, fat: FatPtr) -> &[u8] {
        &self.memory.data(&self.store)[fat.ptr() as usize..(fat.ptr() + fat.len()) as usize]
    }

    pub fn read_string(&self, fat: FatPtr) -> String {
        let slice = self.read(fat);
        str::from_utf8(slice).unwrap().to_string()
    }

    pub fn write(&mut self, ptr: u32, src: &[u8]) {
        self.memory.data_mut(&mut self.store)[ptr as usize..ptr as usize + src.len()]
            .copy_from_slice(src);
    }
}
