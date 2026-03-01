use steel_plugin_sdk::event::{Event, result::EventResult};
use wasmtime::{Instance, Memory, Store};

use crate::{PluginExports, PluginHostData, PluginMeta};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Enabled,
    Disabled,
}

pub struct PluginInstance {
    pub instance: Instance,
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

    pub async fn on_event<T: Event>(
        &mut self,
        handler_name: &str,
        event: &T,
    ) -> Result<EventResult, wasmtime::Error> {
        let event = rmp_serde::to_vec(event).unwrap();
        let len = event.len() as u32;
        let ptr = self.alloc(len).await?;

        self.memory
            .write(&mut self.store, ptr as usize, &event)
            .unwrap();

        let func = self
            .instance
            .get_typed_func(&mut self.store, handler_name)
            .unwrap();
        let result = func.call_async(&mut self.store, (ptr, len)).await.unwrap();

        self.dealloc(ptr, len).await?;
        Ok(EventResult::from_u64(result))
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
