use steel_plugin_sdk::event::EventId;
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
    ) -> Result<u64, wasmtime::Error> {
        let event_id = &(event_id as u16).to_be_bytes();

        let len = (event.len() + event_id.len()) as u32;
        let ptr = self.alloc(len).await?;

        // write event id
        self.memory
            .write(&mut self.store, ptr as usize, event_id)
            .unwrap();
        // write event
        self.memory
            .write(&mut self.store, ptr as usize + event_id.len(), event)
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
