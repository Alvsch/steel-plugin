use std::str::Utf8Error;

use steel_plugin_sdk::utils::fat::FatPtr;
use wasmtime::{AsContext, AsContextMut, Memory};

use crate::PluginState;

pub trait MemoryExt {
    fn read_memory<'a, S>(&'a self, store: &'a S, fat: FatPtr) -> &'a [u8]
    where
        S: AsContext<Data = PluginState>;

    fn write_memory<S>(&self, store: &mut S, ptr: u32, src: &[u8])
    where
        S: AsContextMut<Data = PluginState>;

    fn read_string<S>(&self, store: &S, fat: FatPtr) -> Result<String, Utf8Error>
    where
        S: AsContext<Data = PluginState>,
    {
        let slice = self.read_memory(store, fat);
        str::from_utf8(slice).map(ToString::to_string)
    }
}

impl MemoryExt for Memory {
    fn read_memory<'a, S>(&'a self, store: &'a S, fat: FatPtr) -> &'a [u8]
    where
        S: AsContext<Data = PluginState>,
    {
        &self.data(store)[fat.ptr() as usize..(fat.ptr() + fat.len()) as usize]
    }

    fn write_memory<S>(&self, store: &mut S, ptr: u32, src: &[u8])
    where
        S: AsContextMut<Data = PluginState>,
    {
        self.data_mut(store)[ptr as usize..ptr as usize + src.len()].copy_from_slice(src);
    }
}
