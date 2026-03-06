use std::path::PathBuf;
use steel_host::{PluginHost, discover_plugins};
use tokio::fs::create_dir_all;
use wasmtime::{Config, OptLevel};

#[tokio::main]
async fn main() {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::Speed);

    let plugins_folder = PathBuf::from("plugins");
    create_dir_all(&plugins_folder).await.unwrap();

    let mut host = PluginHost::new(config, plugins_folder.clone()).unwrap();

    let discovered_plugins = discover_plugins(&plugins_folder).await.unwrap();
    for plugin_meta in discovered_plugins {
        let store = host.load_plugin(plugin_meta).await.unwrap();
        store.enable_plugin().await.unwrap();
    }
}
