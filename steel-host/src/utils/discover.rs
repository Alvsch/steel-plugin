use crate::PluginMeta;
use crate::utils::read_custom_section;
use crate::utils::sorting::sort_plugins;
use anyhow::{Context, bail};
use std::path::Path;
use steel_plugin_sdk::STEEL_API_VERSION;
use tokio::fs::{read, read_dir};
use tracing::warn;

/// Discover plugins in the specified directory and return their `PluginMeta` in topological order.
pub async fn discover_plugins(plugin_dir: &Path) -> anyhow::Result<Vec<PluginMeta>> {
    let mut plugins = Vec::new();

    let mut dir = read_dir(plugin_dir)
        .await
        .context("failed to read plugin_dir")?;

    while let Some(entry) = dir.next_entry().await? {
        let file_type = entry.file_type().await?;
        let file_path = entry.path();

        if file_type.is_file() {
            if file_path
                .extension()
                .is_none_or(|x| x.to_str().is_none_or(|ext| ext != "wasm"))
            {
                continue;
            }
            match discover(&file_path).await {
                Ok(plugin_meta) => plugins.push(plugin_meta),
                Err(err) => {
                    warn!("{err}");
                }
            }
        }
    }
    let (topology, invalid) = sort_plugins(plugins);
    if !invalid.is_empty() {
        warn!("plugins with invalid dependencies: {:#?}", invalid);
    }
    Ok(topology)
}

async fn discover(file_path: &Path) -> anyhow::Result<PluginMeta> {
    let bytes = read(&file_path).await.context("failed to read file_path")?;
    let meta_section =
        read_custom_section(&bytes, "plugin_meta")?.context("missing plugin meta")?;

    let mut plugin_meta: PluginMeta =
        rmp_serde::from_slice(meta_section).context("invalid plugin meta")?;

    if plugin_meta.name == "steel" {
        bail!("Skipping plugin 'steel': this name is reserved and cannot be loaded.");
    }

    if plugin_meta.api_version != STEEL_API_VERSION {
        bail!(
            "Plugin '{}' targets API version {} but host is running {}; skipping load",
            plugin_meta.name,
            plugin_meta.api_version,
            STEEL_API_VERSION
        );
    }

    plugin_meta.file_path = file_path.canonicalize()?;
    Ok(plugin_meta)
}
