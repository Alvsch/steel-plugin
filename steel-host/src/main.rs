use steel_host::{
    manager::PluginManager,
    stages::{discover::discover_plugins, resolve::resolve_plugins, setup_lua_vm},
};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let discovered_plugins = discover_plugins("plugins");
    let resolved_plugins = resolve_plugins(discovered_plugins)?;

    let mut manager = PluginManager::new(setup_lua_vm()?);
    manager.load(resolved_plugins)?;

    manager.enable_all()?;
    manager.shutdown()?;

    Ok(())
}
