use steel_host::{
    manager::PluginManager,
    stages::{discover::discover_plugins, resolve::resolve_plugins, setup_lua_vm},
};
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

    let mut manager = PluginManager::new(setup_lua_vm()?);
    manager.load(resolved_plugins)?;

    manager.enable_all()?;
    manager.shutdown()?;

    Ok(())
}
