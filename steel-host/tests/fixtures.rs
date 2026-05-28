#![allow(unused)]
use std::path::PathBuf;

use anyhow::{Context, bail};
use tempfile::TempDir;
use tokio::fs::{copy, create_dir_all};
use wasmtime::{Config, OptLevel};

const FIXTURE_WASM_FILES: [&str; 3] = [
    "provider_plugin.wasm",
    "consumer_plugin.wasm",
    "listening_plugin.wasm",
];

pub struct FixtureLayout {
    _temp: TempDir,
    pub plugin: PathBuf,
    pub data: PathBuf,
}

#[must_use]
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("steel-host should be under the workspace root")
        .to_path_buf()
}

pub async fn setup_layout() -> anyhow::Result<FixtureLayout> {
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

    let artifact_dir = workspace_root().join("target/wasm32-wasip2/profiling");
    for file_name in FIXTURE_WASM_FILES {
        let src = artifact_dir.join(file_name);
        if !src.exists() {
            bail!(
                "missing wasm fixture at '{}'; run `just build-plugin` before integration tests",
                src.display()
            );
        }

        let dst = plugin_dir.join(file_name);
        println!("{}", dst.display());
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

#[must_use]
pub fn host_config() -> Config {
    let mut config = Config::new();
    config.cranelift_opt_level(OptLevel::Speed);
    config.wasm_multi_memory(false);
    config
}
