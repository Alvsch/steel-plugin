use std::{collections::HashMap, sync::Arc};

use mlua::Lua;
use steel_utils::locks::SyncMutex;

use crate::{
    require::PluginRuntime,
    stages::{compile::CompiledPlugin, execute::ExecuteError},
};

pub type PluginRegistry = Arc<SyncMutex<HashMap<String, PluginRuntime>>>;

pub fn build_plugin_registry(
    lua: &Lua,
    plugins: &[CompiledPlugin],
) -> Result<PluginRegistry, ExecuteError> {
    let mut registry = HashMap::with_capacity(plugins.len());

    for plugin in plugins {
        let name = plugin.config.name.clone();

        registry.insert(name.clone(), PluginRuntime::from_compiled(lua, plugin)?);
    }

    Ok(Arc::new(SyncMutex::new(registry)))
}
