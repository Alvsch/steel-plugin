use std::sync::Arc;
use steel_plugin_sdk::{
    event::{Event, result::EventResult},
    utils::fat::FatPtr,
};
use tokio::sync::Mutex;
use wasmtime::{Instance, Memory, Store};

use crate::{PluginExports, PluginMeta, PluginState, utils::memory::PluginMemory};

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
    pub store: Arc<Mutex<Store<PluginState>>>,
}

impl PluginInstance {
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.status == PluginStatus::Enabled
    }

    pub async fn write_to_memory(&mut self, src: &[u8]) -> Result<u32, wasmtime::Error> {
        let ptr = self.alloc(src.len() as u32).await?;
        let mut lock = self.store.lock().await;
        let mut memory = PluginMemory::new(self.memory, &mut *lock);
        memory.write(ptr, src);
        Ok(ptr)
    }

    pub async fn alloc(&mut self, len: u32) -> Result<u32, wasmtime::Error> {
        self.exports
            .alloc
            .call_async(&mut *self.store.lock().await, len)
            .await
    }

    pub async fn dealloc(&mut self, fat: FatPtr) -> Result<(), wasmtime::Error> {
        self.exports
            .dealloc
            .call_async(&mut *self.store.lock().await, (fat.ptr(), fat.len()))
            .await
    }

    pub async fn on_event<T: Event>(
        &mut self,
        handler_name: &str,
        event: &T,
    ) -> Result<EventResult<T>, wasmtime::Error> {
        let mut lock = self.store.lock().await;
        let mut memory = PluginMemory::new(self.memory, &mut *lock);
        let fat = memory.write_msgpack(event, &self.exports.alloc).await;

        let func = self.instance.get_typed_func(&mut *lock, handler_name)?;
        let result: u64 = func.call_async(&mut *lock, (fat.ptr(), fat.len())).await?;
        drop(lock);

        self.dealloc(fat).await?;
        Ok(EventResult::from(result))
    }

    pub async fn enable(&mut self) -> Result<(), wasmtime::Error> {
        self.exports
            .on_enable
            .call_async(&mut *self.store.lock().await, ())
            .await?;
        self.status = PluginStatus::Enabled;
        Ok(())
    }

    pub async fn disable(&mut self) -> Result<(), wasmtime::Error> {
        self.status = PluginStatus::Disabled;
        self.exports
            .on_disable
            .call_async(&mut *self.store.lock().await, ())
            .await
    }
}
