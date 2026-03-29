use crate::PluginState;
use crate::utils::memory::PluginMemory;
use steel_plugin_sdk::rpc::{MethodId, PluginId};
use steel_plugin_sdk::utils::fat::FatPtr;
use tracing::info;
use wasmtime::Caller;
use wasmtime_wasi::p1::wasi_snapshot_preview1;

type HostLinker = wasmtime::Linker<PluginState>;

mod rpc;

pub fn configure_all(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    wasi_snapshot_preview1::add_to_linker(linker, |data: &mut PluginState| &mut data.wasi)?;
    configure_base(linker)?;
    configure_rpc(linker)?;
    Ok(())
}

fn configure_base(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    linker
        .func_wrap(
            "host",
            "info",
            |mut caller: Caller<PluginState>, ptr: u32, len: u32| {
                let exports = caller.data().exports().clone();
                let memory = PluginMemory::new(&mut caller, &exports.memory);
                let buf = memory.read(FatPtr::new(ptr, len).unwrap());
                let message = str::from_utf8(buf).unwrap().to_string();

                let plugin_name = caller.data().meta.name.as_str();
                info!("[{plugin_name}] {message}");
            },
        )?;
    Ok(())
}

fn configure_rpc(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    linker.func_wrap_async(
        "host",
        "rpc_resolve_plugin",
        |caller: Caller<PluginState>, (plugin_name,): (u64,)| {
            Box::new(rpc::resolve_plugin(caller, plugin_name))
        },
    )?;
    linker.func_wrap_async(
        "host",
        "rpc_resolve_method",
        |caller: Caller<PluginState>, (plugin_id, method_name): (PluginId, u64)| {
            Box::new(rpc::resolve_method(caller, plugin_id, method_name))
        },
    )?;
    linker.func_wrap_async(
        "host",
        "rpc_dispatch",
        |caller: Caller<PluginState>,
         (plugin_id, method_id, data_ptr): (PluginId, MethodId, u64)| {
            Box::new(rpc::dispatch(caller, plugin_id, method_id, data_ptr))
        },
    )?;
    Ok(())
}
