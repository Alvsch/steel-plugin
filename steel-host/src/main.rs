use std::path::PathBuf;
use steel_host::event::dispatch_topic;
use steel_host::{PluginHost, discover_plugins};
use steel_plugin_sdk::event::{PlayerJoinEvent, hash_topic};
use tokio::fs::create_dir_all;
use tracing::{Level, info};
use uuid::Uuid;
use wasmtime::{Config, OptLevel};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let mut config = Config::new();
    #[cfg(feature = "profiler")]
    config.profiler(wasmtime::ProfilingStrategy::JitDump);
    config.cranelift_opt_level(OptLevel::Speed);
    config.wasm_multi_memory(false);

    let plugins_folder = PathBuf::from("plugins");
    create_dir_all(&plugins_folder).await.unwrap();

    let mut host = PluginHost::new(config, plugins_folder.clone()).unwrap();

    let discovered_plugins = discover_plugins(&plugins_folder).await.unwrap();
    for plugin_meta in discovered_plugins {
        let store = host.load_plugin(plugin_meta).await.unwrap();
        store.enable_plugin().await.unwrap();
    }

    let mut payload = rmp_serde::to_vec(&PlayerJoinEvent {
        player_id: Uuid::new_v4(),
        username: "Alvsch".to_string(),
    })
    .unwrap();

    let handlers = host.state.handler_registry.read().await;

    dispatch_topic(&handlers, hash_topic(b"PlayerJoinEvent"), &mut payload).await;
    let value: PlayerJoinEvent = rmp_serde::from_slice(&payload).unwrap();
    info!("{:?}", value);
}
