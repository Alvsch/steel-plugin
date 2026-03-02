use crate::PluginState;
use crate::utils::memory::PluginMemory;
use steel_plugin_sdk::event::handler::EventHandler;
use steel_plugin_sdk::utils::fat::FatPtr;
use tracing::info;
use wasmtime::Caller;
use wasmtime_wasi::p1::wasi_snapshot_preview1;

type HostLinker = wasmtime::Linker<PluginState>;

pub fn configure_all(linker: &mut HostLinker) {
    configure_base(linker);
    configure_rpc(linker).unwrap();
}

fn configure_base(linker: &mut HostLinker) {
    wasi_snapshot_preview1::add_to_linker(linker, |data: &mut PluginState| &mut data.wasi).unwrap();
    linker
        .func_wrap(
            "host",
            "info",
            |mut caller: Caller<PluginState>, ptr: u32, len: u32| {
                let memory = PluginMemory::from(&mut caller);
                let buf = memory.read(FatPtr::new(ptr, len).unwrap());
                let message = str::from_utf8(buf).unwrap().to_string();

                let plugin_name = caller.data().name.as_str();
                info!("[{plugin_name}] {message}");
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "register_handler",
            |mut caller: Caller<PluginState>, ptr: u32, len: u32| {
                let memory = PluginMemory::from(&mut caller);
                let handler: EventHandler = memory.read_msgpack(FatPtr::new(ptr, len).unwrap());

                let registry = &*caller.data().registry;
                let plugin_name = caller.data().name.clone();
                registry.register_handler(plugin_name, handler);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "host",
            "register_event",
            |mut caller: Caller<PluginState>, ptr: u32, len: u32| {
                let memory = PluginMemory::from(&mut caller);
                let buf = memory.read(FatPtr::new(ptr, len).unwrap());
                let event_name = str::from_utf8(buf).unwrap().to_string();

                let registry = &*caller.data().registry;
                registry.register_event(event_name);
            },
        )
        .unwrap();
}

fn configure_rpc(linker: &mut HostLinker) -> Result<(), wasmtime::Error> {
    linker.func_wrap_async(
        "host",
        "rpc_register",
        |mut caller: Caller<PluginState>, (export_name,): (u64,)| {
            Box::new(async move {
                let export_name = FatPtr::unpack(export_name).unwrap();
                let memory = PluginMemory::from(&mut caller);

                let export_name = memory.read(export_name);
                let export_name = str::from_utf8(export_name).unwrap().to_string();

                let export = caller
                    .get_export(&export_name)
                    .unwrap()
                    .into_func()
                    .unwrap()
                    .typed::<u64, u64>(&mut caller)
                    .unwrap();

                let PluginState { plugin_id, rpc, .. } = caller.data();
                let plugin_id = *plugin_id;
                let mut rpc = rpc.write().await;
                let method_id = rpc.next_id();
                rpc.get_plugin_mut(plugin_id).unwrap().register_method(
                    method_id,
                    export_name,
                    export,
                );
            })
        },
    )?;
    linker.func_wrap_async(
        "host",
        "rpc_resolve_plugin",
        |mut caller: Caller<PluginState>, (plugin_name_ptr,): (u64,)| {
            Box::new(async move {
                let plugin_name_ptr = FatPtr::unpack(plugin_name_ptr).unwrap();
                let memory = PluginMemory::from(&mut caller);
                let plugin_name = memory.read_string(plugin_name_ptr);

                let rpc = caller.data().rpc.read().await;
                rpc.resolve_plugin(&plugin_name).unwrap()
            })
        },
    )?;
    linker.func_wrap_async(
        "host",
        "rpc_resolve_method",
        |mut caller: Caller<PluginState>, (plugin_id, method_name_ptr): (u32, u64)| {
            Box::new(async move {
                let method_name_ptr = FatPtr::unpack(method_name_ptr).unwrap();
                let memory = PluginMemory::from(&mut caller);
                let method_name = memory.read_string(method_name_ptr);

                let rpc = caller.data().rpc.read().await;
                rpc.resolve_method(plugin_id, &method_name).unwrap()
            })
        },
    )?;
    linker.func_wrap_async(
        "host",
        "rpc_dispatch",
        |mut caller: Caller<PluginState>, (plugin_id, method_id, data_ptr): (u32, u32, u64)| {
            Box::new(async move {
                let data_ptr = FatPtr::unpack(data_ptr).unwrap();
                let memory = PluginMemory::from(&mut caller);
                let data = memory.read(data_ptr).to_vec();

                let rpc = caller.data().rpc.read().await;
                let plugin = rpc.get_plugin(plugin_id).unwrap();
                let method = plugin.get_method(method_id);

                let mut remote_store = plugin.store.lock().await;
                let remote_alloc = remote_store.data().exports.as_ref().unwrap().alloc.clone();

                let len = data.len() as u32;
                let ptr = remote_alloc
                    .call_async(&mut *remote_store, len)
                    .await
                    .unwrap();

                let remote_memory = *remote_store.data().memory.as_ref().unwrap();
                let mut remote_memory = PluginMemory::new(remote_memory, &mut *remote_store);
                remote_memory.write(ptr, &data);

                let fat = FatPtr::new(ptr, len).unwrap();
                method
                    .call_async(&mut *remote_store, fat.pack())
                    .await
                    .unwrap();
            })
        },
    )?;
    Ok(())
}
