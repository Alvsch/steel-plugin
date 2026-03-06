use crate::PluginMeta;
use crate::error::PluginLoaderError;
use crate::utils::read_custom_section;
use crate::utils::sorting::sort_plugins;
use std::path::Path;
use tokio::fs::{read, read_dir};
use tracing::error;

/// Discover plugins in the specified directory and return their `PluginMeta` in topological order.
pub async fn discover_plugins(plugin_dir: &Path) -> Result<Vec<PluginMeta>, PluginLoaderError> {
    let mut plugins = Vec::new();

    let mut dir = read_dir(plugin_dir).await?;
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
            let bytes = read(&file_path).await?;
            let meta_section = read_custom_section(&bytes, "plugin_meta")?
                .ok_or(PluginLoaderError::MissingPluginMeta)?;
            let mut plugin_meta: PluginMeta = rmp_serde::from_slice(meta_section)
                .map_err(PluginLoaderError::InvalidPluginMeta)?;

            if plugin_meta.name == "steel" {
                error!("Skipping plugin 'steel': this name is reserved and cannot be loaded.",);
                continue;
            }

            plugin_meta.file_path = file_path.canonicalize()?;
            plugins.push(plugin_meta);
        }
    }
    let (topology, invalid) = sort_plugins(plugins);
    if !invalid.is_empty() {
        error!("plugins with invalid dependencies: {:#?}", invalid);
    }
    Ok(topology)
}
