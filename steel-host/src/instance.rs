use steel_plugin_sdk::event::Event;
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
        let ptr = self.alloc(buffer.len() as u32).await?;
        self.memory.write(&mut self.store, ptr as usize, buffer)?;
        Ok(ptr)
    }

    pub async fn alloc(&mut self, len: u32) -> Result<u32, wasmtime::Error> {
        self.exports.alloc.call_async(&mut self.store, len).await
    }

    pub async fn dealloc(&mut self, ptr: u32, len: u32) -> Result<(), wasmtime::Error> {
        self.exports
            .dealloc
            .call_async(&mut self.store, (ptr, len))
            .await
    }

    pub async fn on_event<T: Event>(&mut self, event: &T) -> Result<u64, wasmtime::Error> {
        let event = rmp_serde::to_vec(event).unwrap();
        let len = event.len() as u32;
        let ptr = self.alloc(len).await?;

        self.memory
            .write(&mut self.store, ptr as usize, &event)
            .unwrap();

        let result = self
            .exports
            .on_event
            .call_async(&mut self.store, (ptr, len))
            .await?;

        self.dealloc(ptr, len).await?;
        Ok(result)
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
