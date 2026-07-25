use mlua::prelude::*;

use crate::{
    require::registry::PluginRegistry,
    stages::{
        compile::PluginCompiler,
        execute::{LoadedPlugin, execute_plugin},
        resolve::ResolvedPlugin,
    },
};

pub struct PluginManager {
    lua: Lua,
    registry: PluginRegistry,
    plugins: Vec<LoadedPlugin>,
}

impl PluginManager {
    #[must_use]
    pub fn new(lua: Lua) -> Self {
        Self {
            lua,
            registry: PluginRegistry::default(),
            plugins: Vec::new(),
        }
    }

    pub fn load(&mut self, resolved_plugins: Vec<ResolvedPlugin>) -> anyhow::Result<()> {
        let compiler = PluginCompiler::new();

        for plugin in resolved_plugins {
            match compiler.compile(plugin) {
                Ok(compiled_plugin) => {
                    let loaded = execute_plugin(&self.lua, self.registry.clone(), compiled_plugin)?;

                    self.plugins.push(loaded);
                }
                Err(err) => {
                    tracing::warn!(error = %err, "plugin failed to compile");
                }
            }
        }

        Ok(())
    }

    pub fn enable_all(&self) -> anyhow::Result<()> {
        for plugin in &self.plugins {
            plugin.on_enable(&self.lua)?;
        }

        Ok(())
    }

    pub fn shutdown(mut self) -> anyhow::Result<()> {
        while let Some(plugin) = self.plugins.pop() {
            plugin.on_disable(&self.lua)?;

            if let Some(runtime) = self.registry.lock().remove(&plugin.name) {
                runtime.cleanup(&self.lua)?;
            }

            plugin.cleanup(&self.lua)?;
        }

        Ok(())
    }
}
