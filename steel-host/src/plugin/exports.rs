use steel_plugin_sdk::utils::fat::FatPtr;
use wasmtime::{Instance, Memory, Store, TypedFunc};

use crate::{PluginState, error::PluginContractError};

pub type AllocFunc = TypedFunc<u32, u32>;
pub type DeallocFunc = TypedFunc<(u32, u32), ()>;

pub struct PluginExports {
    /// (`ptr`, `len`)
    pub alloc: AllocFunc,
    /// (`ptr`, `len`)
    pub dealloc: DeallocFunc,
    on_load: TypedFunc<(), u64>,
    on_enable: TypedFunc<(), ()>,
    on_disable: TypedFunc<(), ()>,
    pub memory: Memory,
    pub instance: Instance,
}

impl PluginExports {
    pub fn resolve(
        instance: Instance,
        store: &mut Store<PluginState>,
    ) -> Result<PluginExports, PluginContractError> {
        Ok(Self {
            alloc: instance
                .get_typed_func(&mut *store, "alloc")
                .map_err(|err| PluginContractError::InvalidExport {
                    name: "alloc",
                    reason: err.to_string().into(),
                })?,
            dealloc: instance
                .get_typed_func(&mut *store, "dealloc")
                .map_err(|err| PluginContractError::InvalidExport {
                    name: "dealloc",
                    reason: err.to_string().into(),
                })?,
            on_load: instance
                .get_typed_func(&mut *store, "on_load")
                .map_err(|err| PluginContractError::InvalidExport {
                    name: "on_load",
                    reason: err.to_string().into(),
                })?,
            on_enable: instance
                .get_typed_func(&mut *store, "on_enable")
                .map_err(|err| PluginContractError::InvalidExport {
                    name: "on_enable",
                    reason: err.to_string().into(),
                })?,
            on_disable: instance
                .get_typed_func(&mut *store, "on_disable")
                .map_err(|err| PluginContractError::InvalidExport {
                    name: "on_disable",
                    reason: err.to_string().into(),
                })?,
            memory: instance.get_memory(&mut *store, "memory").ok_or(
                PluginContractError::InvalidExport {
                    name: "memory",
                    reason: "memory missing".into(),
                },
            )?,
            instance,
        })
    }

    pub async fn alloc(
        &self,
        store: &mut Store<PluginState>,
        len: u32,
    ) -> Result<FatPtr, PluginContractError> {
        let ptr = self.alloc.call_async(store, len).await?;
        let fat = FatPtr::new(ptr, len).ok_or(PluginContractError::NullAllocation)?;
        Ok(fat)
    }

    pub async fn dealloc(
        &self,
        store: &mut Store<PluginState>,
        fat: FatPtr,
    ) -> Result<(), PluginContractError> {
        self.dealloc.call_async(store, (fat.ptr(), fat.len())).await?;
        Ok(())
    }

    pub async fn on_load(
        &self,
        store: &mut Store<PluginState>,
    ) -> Result<FatPtr, PluginContractError> {
        let packed = self.on_load.call_async(store, ()).await?;
        let fat = FatPtr::unpack(packed).ok_or(PluginContractError::NullLoadData)?;
        Ok(fat)
    }

    pub async fn on_enable(
        &self,
        store: &mut Store<PluginState>,
    ) -> Result<(), PluginContractError> {
        self.on_enable.call_async(store, ()).await?;
        Ok(())
    }

    pub async fn on_disable(
        &self,
        store: &mut Store<PluginState>,
    ) -> Result<(), PluginContractError> {
        self.on_disable.call_async(store, ()).await?;
        Ok(())
    }
}
