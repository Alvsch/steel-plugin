use steel_plugin_sdk::event::{EventId, EventResult};
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

    pub async fn on_event(
        &mut self,
        event_id: EventId,
        event: &[u8],
    ) -> Result<EventResult, wasmtime::Error> {
        let event_len = event.len() as u32;
        let event_ptr = self.write_to_memory(&event).await?;

        let result = self
            .exports
            .on_event
            .call_async(&mut self.store, (event_id as u32, event_ptr, event_len))
            .await?;

        self.dealloc(event_ptr, event_len).await?;
        Ok(EventResult::from_bits_truncate(result as u8))
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
