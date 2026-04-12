use anyhow::Context;
use std::path::PathBuf;
use std::sync::Arc;
use steel_host::{PluginHost, discover_plugins};
use tokio::fs::create_dir_all;
use tracing_subscriber::EnvFilter;
use wasmtime::{Config, OptLevel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi_sanitization(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive("debug".parse()?)
                .from_env_lossy()
                .add_directive("cranelift_codegen=info".parse()?)
                .add_directive("wasmtime_internal_cranelift::compiler=info".parse()?),
        )
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

    for plugin in enabled.drain(..).rev() {
        host.state
            .disable_plugin(&plugin)
            .await
            .context("failed to disable plugin")?;
    }

    Ok(())
}
