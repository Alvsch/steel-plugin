use std::{path::PathBuf, sync::Arc};
use steel_host::{EventRegistry, PluginLoader, PluginManager, configure_linker};
use steel_plugin_sdk::event::PlayerJoinEvent;
use tokio::fs::create_dir_all;
use tracing::info;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use wasmtime::{Config, Linker, OptLevel};

#[tokio::main]
async fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
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
    configure_linker(&mut linker);

    let plugins_path = path.join("plugins");
    create_dir_all(&plugins_path).await.unwrap();

    let registry = Arc::new(EventRegistry::new());
    let loader = PluginLoader::new(engine, linker, plugins_path, registry.clone());
    let mut manager = PluginManager::new();

    let discovered_plugins = loader
        .discover_plugins(&path.join("target/wasm32-wasip1/debug/"))
        .await
        .unwrap();

    let mut loaded_plugins = Vec::new();
    for plugin_meta in discovered_plugins {
        let loaded_plugin = loader.load_plugin(plugin_meta).await.unwrap();
        loaded_plugins.push(loaded_plugin);
    }

    manager.add_all(loaded_plugins);
    manager.enable_all().await;

    let event = PlayerJoinEvent {
        cancelled: false,
        player: Uuid::new_v4(),
    };

    info!("old: {event:#?}");
    let new_event = registry.call_event(&mut manager, event).await;
    info!("new: {new_event:#?}");

    manager.disable_all().await;
    manager.clear();
}
