use anyhow::Context;
use glam::DVec3;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use steel_host::{
    PluginHost, discover_plugins,
    objects::demo_player::{DemoPlayer, make_player_handler},
};
use steel_plugin_sdk::event::PlayerJoinEvent;
use steel_plugin_sdk::objects::player::Player;
use steel_plugin_sdk::objects::{GameType, Handle, HandleKey};
use tokio::fs::create_dir_all;
use tracing::{Level, info};
use wasmtime::{Config, OptLevel};

async fn register_demo_player(host: &PluginHost, player: DemoPlayer) -> HandleKey {
    let player = Arc::new(Mutex::new(player));
    host.state
        .register_object_handler(make_player_handler(player))
        .await
}

async fn dispatch_demo_join_event(host: &PluginHost, key: HandleKey) -> anyhow::Result<()> {
    let mut event = PlayerJoinEvent {
        player: Handle::<Player>::from_raw(key),
    };

    let handlers = host.state.handler_registry.read().await;
    handlers
        .dispatch_topic(&mut event)
        .await
        .context("failed to dispatch topic")?;

    info!("modified {:?}", event);
    Ok(())
}

async fn unregister_demo_player(host: &PluginHost, key: HandleKey) {
    host.state.unregister_object_handler(key).await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi_sanitization(false)
        .with_max_level(Level::INFO)
        .init();

    let mut config = Config::new();
    #[cfg(feature = "profiler")]
    config.profiler(wasmtime::ProfilingStrategy::JitDump);
    config.cranelift_opt_level(OptLevel::Speed);
    config.wasm_multi_memory(false);

    let plugins_folder = PathBuf::from("plugins");
    create_dir_all(&plugins_folder)
        .await
        .context("failed to create plugin directory")?;

    let host = Arc::new(
        PluginHost::new(config, plugins_folder.clone()).expect("failed to create PluginHost"),
    );

    let player_key = register_demo_player(
        host.as_ref(),
        DemoPlayer {
            name: "Steve".to_string(),
            health: 20.0,
            position: DVec3::ZERO,
            gamemode: GameType::Survival,
        },
    )
    .await;

    let discovered_plugins = discover_plugins(&plugins_folder)
        .await
        .context("failed to discover plugins")?;

    let mut plugins = Vec::new();
    for plugin_meta in discovered_plugins {
        let cloned = host.clone();
        plugins.push(tokio::spawn(async move {
            cloned.prepare_plugin(plugin_meta).await
        }));
    }

    let mut enabled = Vec::new();
    for handle in plugins.drain(..) {
        let plugin = handle
            .await
            .context("tokio thread panicked")?
            .context("failed to prepare plugin")?;

        host.load_plugin(&plugin)
            .await
            .context("failed to load plugin")?;

        host.enable_plugin(&plugin)
            .await
            .context("failed to enable plugin")?;

        enabled.push(plugin);
    }

    dispatch_demo_join_event(host.as_ref(), player_key).await?;

    for plugin in enabled.drain(..).rev() {
        host.state
            .disable_plugin(&plugin)
            .await
            .context("failed to disable plugin")?;
    }

    unregister_demo_player(host.as_ref(), player_key).await;

    Ok(())
}
