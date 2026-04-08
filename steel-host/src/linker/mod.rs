use std::num::NonZeroU32;

use crate::PluginState;
use crate::error::PluginContractError;
use crate::utils::memory::PluginMemory;
use steel_plugin_sdk::objects::HandleKey;
use steel_plugin_sdk::utils::fat::FatPtr;
use tracing::Level;
use wasmtime::Caller;
use wasmtime_wasi::p1::wasi_snapshot_preview1;

type HostLinker = wasmtime::Linker<PluginState>;

mod objects;
mod rpc;

pub fn configure_all(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    wasi_snapshot_preview1::add_to_linker(linker, |data: &mut PluginState| &mut data.wasi)?;
    configure_logging(linker)?;
    configure_rpc(linker)?;
    configure_objects(linker)?;
    Ok(())
}

fn register_log_import(
    linker: &mut HostLinker,
    import_name: &'static str,
    level: Level,
) -> Result<(), wasmtime::Error> {
    linker.func_wrap(
        "host",
        import_name,
        move |mut caller: Caller<PluginState>, ptr: u32, len: u32| -> Result<(), wasmtime::Error> {
            let exports = caller.data().exports().clone();
            let memory = PluginMemory::new(&mut caller, &exports.memory);
            let fat = FatPtr::new(ptr, len).ok_or(PluginContractError::NullPointer)?;
            let buf = memory.read(fat);
            let message = str::from_utf8(buf)?.to_string();
            let plugin_name = caller.data().meta.name.as_str();
            match level {
                Level::ERROR => tracing::error!("[{plugin_name}] {message}"),
                Level::WARN => tracing::warn!("[{plugin_name}] {message}"),
                Level::INFO => tracing::info!("[{plugin_name}] {message}"),
                Level::DEBUG => tracing::debug!("[{plugin_name}] {message}"),
                Level::TRACE => tracing::trace!("[{plugin_name}] {message}"),
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn configure_logging(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    register_log_import(linker, "error", Level::ERROR)?;
    register_log_import(linker, "warn", Level::WARN)?;
    register_log_import(linker, "info", Level::INFO)?;
    register_log_import(linker, "debug", Level::DEBUG)?;
    register_log_import(linker, "trace", Level::TRACE)?;
    Ok(())
}

fn configure_rpc(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    linker.func_wrap_async(
        "host",
        "rpc_resolve_plugin",
        |caller: Caller<PluginState>, (plugin_name,): (u64,)| {
            Box::new(async move {
                let plugin_name =
                    FatPtr::unpack(plugin_name).ok_or(PluginContractError::NullPointer)?;
                let plugin_id = rpc::resolve_plugin(caller, plugin_name).await?;
                Ok(plugin_id.map_or(0, NonZeroU32::get))
            })
        },
    )?;
    linker.func_wrap_async(
        "host",
        "rpc_resolve_method",
        |caller: Caller<PluginState>, (plugin_id, method_name): (u32, u64)| {
            Box::new(async move {
                let plugin_id = NonZeroU32::new(plugin_id).ok_or(PluginContractError::InvalidId)?;
                let method_name =
                    FatPtr::unpack(method_name).ok_or(PluginContractError::NullPointer)?;
                let method_id = rpc::resolve_method(caller, plugin_id, method_name).await?;
                Ok(method_id.map_or(0, NonZeroU32::get))
            })
        },
    )?;
    linker.func_wrap_async(
        "host",
        "rpc_dispatch",
        |caller: Caller<PluginState>, (plugin_id, method_id, data_ptr): (u32, u32, u64)| {
            Box::new(async move {
                let plugin_id = NonZeroU32::new(plugin_id).ok_or(PluginContractError::InvalidId)?;
                let method_id = NonZeroU32::new(method_id).ok_or(PluginContractError::InvalidId)?;
                let data_ptr = FatPtr::unpack(data_ptr).ok_or(PluginContractError::NullPointer)?;
                let result = rpc::dispatch(caller, plugin_id, method_id, data_ptr).await?;
                Ok(result.map_or(0, FatPtr::pack))
            })
        },
    )?;
    Ok(())
}

fn configure_objects(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    linker.func_wrap_async(
        "host",
        "object_fetch",
        |caller: Caller<PluginState>, (entity_key, queries_ptr, queries_len): (u64, u32, u32)| {
            Box::new(async move {
                let queries_fat = FatPtr::new(queries_ptr, queries_len)
                    .ok_or(PluginContractError::NullPointer)?;
                let entity_key = HandleKey::from_ffi(entity_key);
                let result = objects::fetch(caller, entity_key, queries_fat).await?;
                Ok(result)
            })
        },
    )?;

    linker.func_wrap_async(
        "host",
        "object_batch_dispatch",
        |caller: Caller<PluginState>, (entity_key, ptr, len): (u64, u32, u32)| {
            Box::new(async move {
                let commands_fat = FatPtr::new(ptr, len).ok_or(PluginContractError::NullPointer)?;
                let entity_key = HandleKey::from_ffi(entity_key);
                objects::batch_dispatch(caller, entity_key, commands_fat).await?;
                Ok(())
            })
        },
    )?;

    Ok(())
}
