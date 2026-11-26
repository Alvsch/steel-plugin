use crate::utils::read_custom_section;
use crate::utils::sorting::sort_plugins;
use anyhow::{Context, bail};
use semver::Version;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use steel_plugin_core::PluginMeta;
use steel_plugin_core::STEEL_API_VERSION;
use tokio::fs::{read, read_dir};
use tracing::warn;

/// A plugin's metadata and the path to its WebAssembly component.
#[derive(Debug)]
pub struct DiscoveredPlugin {
    /// Metadata read from the plugin component.
    pub meta: PluginMeta,
    /// Canonical path to the component file.
    pub path: PathBuf,
}

/// Discover plugins in a directory, ordered so dependencies precede dependents.
pub async fn discover_plugins(plugin_dir: &Path) -> anyhow::Result<Vec<DiscoveredPlugin>> {
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
                Ok(plugin_meta) => plugins.push(DiscoveredPlugin {
                    meta: plugin_meta,
                    path: file_path.canonicalize()?,
                }),
                Err(err) => {
                    warn!("{err}");
                }
            }
        }
    }

    let mut plugins = plugins
        .into_iter()
        .map(|plugin| (plugin.meta.name.clone(), plugin))
        .collect::<HashMap<String, DiscoveredPlugin>>();

    let (topology, invalid) = sort_plugins(
        plugins
            .iter()
            .map(|(name, plugin)| (name.clone(), plugin.meta.depends.clone()))
            .collect(),
    );
    if !invalid.is_empty() {
        warn!("plugins with invalid dependencies: {:#?}", invalid);
    }

    let topology = topology
        .into_iter()
        .filter_map(|name| plugins.remove(&name))
        .collect();

    Ok(topology)
}

async fn discover(file_path: &Path) -> anyhow::Result<PluginMeta> {
    let bytes = read(&file_path).await.context("failed to read file_path")?;
    let meta_section = read_custom_section(&bytes, "steel-api::plugin::metadata")?
        .context("missing plugin meta")?;

    let plugin_meta: PluginMeta =
        rmp_serde::from_slice(meta_section).context("invalid plugin meta")?;

    if plugin_meta.name == "steel" {
        bail!("Skipping plugin 'steel': this name is reserved and cannot be loaded.");
    }

    if !is_compatible_api_version(&plugin_meta.api_version) {
        bail!(
            "Plugin '{}' targets API version {} but host is running {}; skipping load",
            plugin_meta.name,
            plugin_meta.api_version,
            STEEL_API_VERSION
        );
    }

    Ok(plugin_meta)
}

pub fn is_compatible_api_version(version: &Version) -> bool {
    version.major == STEEL_API_VERSION.major && version.minor <= STEEL_API_VERSION.minor
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use steel_plugin_core::STEEL_API_VERSION;

    use super::is_compatible_api_version;

    #[test]
    fn accepts_same_major_with_equal_or_lower_minor() {
        let lower_minor = Version::new(STEEL_API_VERSION.major, 0, 999);

        assert!(is_compatible_api_version(&STEEL_API_VERSION));
        assert!(is_compatible_api_version(&lower_minor));
    }

    #[test]
    fn rejects_same_major_with_higher_minor() {
        let higher_minor = Version::new(STEEL_API_VERSION.major, STEEL_API_VERSION.minor + 1, 0);
        assert!(!is_compatible_api_version(&higher_minor));
    }

    #[test]
    fn rejects_different_major() {
        let different_major = Version::new(STEEL_API_VERSION.major + 1, 0, 0);
        assert!(!is_compatible_api_version(&different_major));
    }
}
