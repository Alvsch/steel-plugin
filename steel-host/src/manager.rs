use indexmap::IndexMap;
use thiserror::Error;
use tracing::{debug, error, warn};

use crate::LoadedPlugin;

#[derive(Debug, Error)]
pub enum PluginManagerError {
    #[error("plugin not found")]
    PluginNotFound,
    #[error("already enabled")]
    AlreadyEnabled,
    #[error("already disabled")]
    AlreadyDisabled,
    #[error("depends on '{dependency}' which is not enabled")]
    MissingDependency { dependency: String },
    #[error("still depended on by: {dependents:?}")]
    StillDependent { dependents: Box<[String]> },
    #[error("still enabled")]
    StillEnabled,
    #[error("wasmtime: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}

pub struct PluginManager {
    plugins: IndexMap<String, LoadedPlugin>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: IndexMap::new(),
        }
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &LoadedPlugin)> {
        self.plugins.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn add(&mut self, plugin: LoadedPlugin) {
        let name = plugin.store.data().plugin_meta.name.clone();
        self.plugins.insert(name, plugin);
    }

    pub fn add_all(&mut self, plugins: impl IntoIterator<Item = LoadedPlugin>) {
        for plugin in plugins {
            self.add(plugin);
        }
    }

    pub fn remove(&mut self, name: &str) {
        let plugin = self.plugins.get(name).unwrap();
        if plugin.enabled {
            warn!("cannot remove enabled plugin '{name}', disable it first");
            return;
        }
        self.plugins.shift_remove(name);
    }

    pub fn clear(&mut self) {
        self.plugins.clear();
    }

    pub async fn enable(&mut self, name: &str) -> Result<(), PluginManagerError> {
        let plugin = self
            .plugins
            .get(name)
            .ok_or(PluginManagerError::PluginNotFound)?;

        if plugin.enabled {
            return Err(PluginManagerError::AlreadyEnabled);
        }

        for dep in &plugin.metadata().depends {
            let dep_missing = self.plugins.get(dep.as_str()).is_none_or(|x| !x.enabled);
            if dep_missing {
                return Err(PluginManagerError::MissingDependency {
                    dependency: dep.clone(),
                });
            }
        }

        debug!("Enabling {name}!");

        let plugin = self.plugins.get_mut(name).unwrap();
        plugin
            .exports
            .on_enable
            .call_async(&mut plugin.store, ())
            .await?;
        plugin.enabled = true;

        Ok(())
    }

    pub async fn disable(&mut self, name: &str) -> Result<(), PluginManagerError> {
        let plugin = self
            .plugins
            .get(name)
            .ok_or(PluginManagerError::PluginNotFound)?;

        if !plugin.enabled {
            return Err(PluginManagerError::AlreadyDisabled);
        }

        let dependents: Box<[String]> = self
            .plugins
            .iter()
            .filter(|(_, p)| {
                p.enabled && p.store.data().plugin_meta.depends.iter().any(|d| d == name)
            })
            .map(|(k, _)| k.clone())
            .collect();

        if !dependents.is_empty() {
            return Err(PluginManagerError::StillDependent { dependents });
        }

        debug!("Disabling {name}!");

        let plugin = self.plugins.get_mut(name).unwrap();
        plugin.enabled = false;
        plugin
            .exports
            .on_disable
            .call_async(&mut plugin.store, ())
            .await?;

        Ok(())
    }

    pub async fn enable_all(&mut self) {
        let names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in names {
            if let Err(err) = self.enable(&name).await {
                self.remove(&name);
                error!("{err}");
            }
        }
    }

    pub async fn disable_all(&mut self) {
        let names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in names {
            if let Err(err) = self.disable(&name).await {
                error!("{err}");
            }
        }
    }
}
