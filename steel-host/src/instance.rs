use wasmtime::{Memory, Store};

use crate::{PluginExports, PluginHostData, PluginMeta};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Enabled,
    Disabled,
}

pub struct PluginInstance {
    pub meta: PluginMeta,
    pub status: PluginStatus,
    pub exports: PluginExports,
    pub memory: Memory,
    pub store: Store<PluginHostData>,
}

impl PluginInstance {
    pub fn is_enabled(&self) -> bool {
        self.status == PluginStatus::Enabled
    }

    pub async fn write_to_memory(&mut self, buffer: &[u8]) -> Result<u32, wasmtime::Error> {
        let ptr = self
            .exports
            .alloc
            .call_async(&mut self.store, buffer.len() as u32)
            .await?;
        self.memory.write(&mut self.store, ptr as usize, buffer)?;
        Ok(ptr)
    }

    pub async fn dealloc(&mut self, ptr: u32, len: u32) -> Result<(), wasmtime::Error> {
        self.exports
            .dealloc
            .call_async(&mut self.store, (ptr, len))
            .await
    }

    pub async fn enable(&mut self) -> Result<(), wasmtime::Error> {
        self.exports
            .on_enable
            .call_async(&mut self.store, ())
            .await?;
        self.status = PluginStatus::Enabled;
        Ok(())
    }

    pub async fn disable(&mut self) -> Result<(), wasmtime::Error> {
        self.status = PluginStatus::Disabled;
        self.exports
            .on_disable
            .call_async(&mut self.store, ())
            .await
    }
}
