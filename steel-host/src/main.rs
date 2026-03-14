use std::path::PathBuf;
use steel_host::event::dispatch_topic;
use steel_host::{PluginHost, discover_plugins};
use steel_plugin_sdk::event::PlayerJoinEvent;
use tokio::fs::create_dir_all;
use tracing::Level;
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

    let event = PlayerJoinEvent {
        player_id: 42,
        username: "Alvsch".to_string(),
    };
    let payload = rmp_serde::to_vec(&event).unwrap();
    let topic_id = 0;
    let handlers = host.state.handler_registry.read().await;
    dispatch_topic(&handlers, topic_id, &payload).await;
}
