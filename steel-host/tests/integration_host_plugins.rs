use std::path::PathBuf;

use anyhow::{Context, bail};
use steel_host::wasmtime::{Config, OptLevel};
use steel_host::{PluginHost, discover_plugins};
use tempfile::TempDir;
use tokio::fs::{copy, create_dir_all};

const FIXTURE_WASM_FILES: [&str; 3] = [
    "provider_plugin.wasm",
    "consumer_plugin.wasm",
    "listening_plugin.wasm",
];

struct FixtureLayout {
    _temp: TempDir,
    plugin: PathBuf,
    data: PathBuf,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("steel-host should be under the workspace root")
        .to_path_buf()
}

async fn setup_fixture_layout() -> anyhow::Result<FixtureLayout> {
    let temp_dir = TempDir::new().context("failed to create temporary directory")?;
    let root = temp_dir.path().to_path_buf();
    let plugin_dir = root.join("plugins");
    let data_dir = root.join("data");

    create_dir_all(&plugin_dir)
        .await
        .context("failed to create plugin fixture directory")?;
    create_dir_all(&data_dir)
        .await
        .context("failed to create plugin data directory")?;

    let artifact_dir = workspace_root().join("target/wasm32-wasip1/profiling");
    for file_name in FIXTURE_WASM_FILES {
        let src = artifact_dir.join(file_name);
        if !src.exists() {
            bail!(
                "missing wasm fixture at '{}'; run `just build-plugin` before integration tests",
                src.display()
            );
        }

        let dst = plugin_dir.join(file_name);
        copy(&src, &dst).await.with_context(|| {
            format!(
                "failed to copy wasm fixture from '{}' to '{}'",
                src.display(),
                dst.display()
            )
        })?;
    }

    Ok(FixtureLayout {
        _temp: temp_dir,
        plugin: plugin_dir,
        data: data_dir,
    })
}

fn host_config() -> Config {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::Speed);
    config.wasm_multi_memory(false);
    config
}

#[tokio::test]
async fn discover_orders_provider_before_consumer() -> anyhow::Result<()> {
    let fixture = setup_fixture_layout().await?;

    let discovered = discover_plugins(&fixture.plugin).await?;
    let names: Vec<String> = discovered.into_iter().map(|meta| meta.name).collect();

    let provider_index = names
        .iter()
        .position(|name| name == "provider-plugin")
        .context("provider-plugin was not discovered")?;
    let consumer_index = names
        .iter()
        .position(|name| name == "consumer-plugin")
        .context("consumer-plugin was not discovered")?;

    assert!(
        provider_index < consumer_index,
        "provider-plugin should be discovered before consumer-plugin; order={names:?}"
    );
    assert!(
        names.iter().any(|name| name == "listening-plugin"),
        "listening-plugin should be discovered; order={names:?}"
    );

    Ok(())
}

#[tokio::test]
async fn lifecycle_load_enable_disable_all_fixtures() -> anyhow::Result<()> {
    let fixture = setup_fixture_layout().await?;
    let discovered = discover_plugins(&fixture.plugin).await?;

    assert!(
        !discovered.is_empty(),
        "expected at least one plugin fixture to be discovered"
    );

    let host = PluginHost::new(host_config(), fixture.data.clone())
        .map_err(|err| anyhow::anyhow!("failed to construct PluginHost: {err}"))?;

    let plugin_names: Vec<String> = discovered.iter().map(|meta| meta.name.clone()).collect();
    let mut enabled_plugins = Vec::new();

    for plugin_meta in discovered {
        let plugin = host
            .prepare_plugin(plugin_meta)
            .await
            .context("failed to prepare plugin")?;
        host.load_plugin(&plugin)
            .await
            .context("failed to load plugin")?;
        host.enable_plugin(&plugin)
            .await
            .context("failed to enable plugin")?;
        enabled_plugins.push(plugin);
    }

    for plugin_name in &plugin_names {
        assert!(
            host.state.resolve_plugin(plugin_name).await.is_some(),
            "plugin '{plugin_name}' should be registered after load/enable"
        );
    }

    while let Some(plugin) = enabled_plugins.pop() {
        host.disable_plugin(&plugin)
            .await
            .context("failed to disable plugin")?;
    }

    for plugin_name in &plugin_names {
        assert!(
            host.state.resolve_plugin(plugin_name).await.is_none(),
            "plugin '{plugin_name}' should be unregistered after disable"
        );
    }

    Ok(())
}
