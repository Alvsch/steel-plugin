use std::path::PathBuf;
use steel_host::{PluginHostData, PluginLoader, PluginManager};
use tokio::fs::create_dir_all;
use tracing::info;
use tracing_subscriber::EnvFilter;
use wasmtime::{Caller, Config, Linker, OptLevel};
use wasmtime_wasi::p1::wasi_snapshot_preview1::add_to_linker;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(
            "debug,cranelift_codegen=info,wasmtime_internal_cranelift=info",
        ))
        .init();

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

                let plugin_name = caller.data().name.as_str();
                info!("[{plugin_name}] {message}");
            },
        )
        .unwrap();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let plugins_path = path.join("plugins");
    create_dir_all(&plugins_path).await.unwrap();

    let loader = PluginLoader::new(engine, linker, plugins_path);
    let mut manager = PluginManager::new();

    let discovered_plugins = loader
        .discover_plugins(&path.join("target/wasm32-wasip1/debug/"))
        .await
        .unwrap();

    println!("{:?}", discovered_plugins);

    let mut loaded_plugins = Vec::new();
    for plugin_meta in discovered_plugins {
        let loaded_plugin = loader.load_plugin(plugin_meta).await.unwrap();
        loaded_plugins.push(loaded_plugin);
    }

    manager.add_all(loaded_plugins);
    manager.enable_all().await;
    manager.disable_all().await;
    manager.clear();
}
