use std::sync::Arc;

use indexmap::IndexMap;
use parking_lot::RwLock;
use thiserror::Error;
use tracing::{debug, error, warn};

use crate::rpc::HostRpc;
use crate::{EventRegistry, instance::PluginInstance};

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
    plugins: IndexMap<String, PluginInstance>,
    pub rpc: Arc<RwLock<HostRpc>>,
    pub registry: Arc<EventRegistry>,
}

impl PluginManager {
    #[must_use]
    pub fn new(registry: Arc<EventRegistry>) -> Self {
        Self {
            plugins: IndexMap::new(),
            rpc: Arc::new(RwLock::new(HostRpc::new())),
            registry,
        }
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PluginInstance> {
        self.plugins.get(name)
    }

    #[must_use]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut PluginInstance> {
        self.plugins.get_mut(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &PluginInstance)> {
        self.plugins.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn add(&mut self, plugin: PluginInstance) {
        let name = plugin.meta.name.clone();
        if name == "steel" {
            error!("Skipping plugin 'steel': this name is reserved and cannot be loaded.",);
            return;
        }

        self.plugins.insert(name, plugin);
    }

    pub fn add_all(&mut self, plugins: impl IntoIterator<Item = PluginInstance>) {
        for plugin in plugins {
            self.add(plugin);
        }
    }

    pub fn remove(&mut self, name: &str) {
        let plugin = self.plugins.get(name).unwrap();
        if plugin.is_enabled() {
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

        if plugin.is_enabled() {
            return Err(PluginManagerError::AlreadyEnabled);
        }

        for dep in &plugin.meta.depends {
            let dep_missing = self
                .plugins
                .get(dep.as_str())
                .is_none_or(|x| !x.is_enabled());
            if dep_missing {
                return Err(PluginManagerError::MissingDependency {
                    dependency: dep.clone(),
                });
            }
        }

        debug!("Enabling {name}!");

        let plugin = self.plugins.get_mut(name).unwrap();
        plugin.enable().await?;
        Ok(())
    }

    pub async fn disable(&mut self, name: &str) -> Result<(), PluginManagerError> {
        let plugin = self
            .plugins
            .get(name)
            .ok_or(PluginManagerError::PluginNotFound)?;

        if !plugin.is_enabled() {
            return Err(PluginManagerError::AlreadyDisabled);
        }

        let dependents: Box<[String]> = self
            .plugins
            .iter()
            .filter(|(_, plugin)| {
                plugin.is_enabled() && plugin.meta.depends.iter().any(|dep| dep == name)
            })
            .map(|(k, _)| k.clone())
            .collect();

        if !dependents.is_empty() {
            return Err(PluginManagerError::StillDependent { dependents });
        }

        debug!("Disabling {name}!");

        let plugin = self.plugins.get_mut(name).unwrap();
        plugin.disable().await?;
        self.rpc.write().unregister_plugin(name);
        self.registry.unregister_handlers(name);
        Ok(())
    }

    // Enables all plugins in order
    pub async fn enable_all(&mut self) {
        let names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in names {
            if let Err(err) = self.enable(&name).await {
                self.remove(&name);
                error!("{err}");
            }
        }
    }

    /// Disables all plugins in reverse order
    pub async fn disable_all(&mut self) {
        let names: Vec<String> = self.plugins.keys().cloned().rev().collect();
        for name in names {
            if let Err(err) = self.disable(&name).await {
                error!("{err}");
            }
        }
    }
}
