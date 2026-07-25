use std::{collections::HashMap, sync::Arc};

use steel_host::stages::{
    compile::PluginCompiler, discover::discover_plugins, execute::execute_plugin,
    resolve::resolve_plugins, setup_lua_vm,
};
use steel_utils::locks::SyncMutex;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // discover - find plugins in a directory with a valid config.toml
    // resolve - further validate the config.toml version requirements, topo sort and build file table
    // compile - compile plugin into bytecode
    // execute - evaluates the init.lua file and returns on_enable/on_disable
    // enable - runs the on_enable function
    // runtime
    // disable - runs the on_disable function
    // teardown - unload the plugin and delete it's env

    let discovered_plugins = discover_plugins("examples");
    let resolved_plugins = resolve_plugins(discovered_plugins)?;
    let compiler = PluginCompiler::new();

    let mut compiled_plugins = Vec::with_capacity(resolved_plugins.len());
    for plugin in resolved_plugins {
        let name = plugin.config.name.clone();
        match compiler.compile(plugin) {
            Ok(compiled) => compiled_plugins.push(compiled),
            Err(err) => {
                tracing::warn!(plugin = %name, error = %err, "plugin failed to compile, skipping");
            }
        }
    }
    let lua = setup_lua_vm()?;
    let registry = Arc::new(SyncMutex::new(HashMap::new()));

    let mut loaded_plugins = Vec::with_capacity(compiled_plugins.len());
    for plugin in compiled_plugins {
        let loaded = execute_plugin(&lua, registry.clone(), plugin)?;
        loaded_plugins.push(loaded);
    }

    for plugin in loaded_plugins {
        plugin.on_enable(&lua)?;
    }

    Ok(())
}
