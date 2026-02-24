use std::path::PathBuf;

use steel_host::{PluginHostData, PluginLoader};
use tracing::info;
use wasmtime::{Caller, Config, Linker, OptLevel};
use wasmtime_wasi::p1::wasi_snapshot_preview1::add_to_linker;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::Speed);
    config.wasm_multi_memory(false);

    let engine = wasmtime::Engine::new(&config).unwrap();
    let mut linker = Linker::new(&engine);
    add_to_linker(&mut linker, |data: &mut PluginHostData| &mut data.wasi).unwrap();
    linker
        .func_wrap(
            "host",
            "info",
            |mut caller: Caller<PluginHostData>, ptr: u32, len: u32| {
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let buf = &memory.data(&caller)[ptr as usize..ptr as usize + len as usize];
                let message = str::from_utf8(buf).unwrap();

                let plugin_name = caller.data().plugin_name.as_str();
                info!("[{plugin_name}] {message}");
            },
        )
        .unwrap();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let loader = PluginLoader::new(engine, linker, path.join("plugins"));

    let plugins = loader
        .discover_plugins(&path.join("target/wasm32-wasip1/debug/"))
        .await;
    
    for plugin in plugins {
        loader.load_plugin(plugin).await;
    }
}
