use steel_host::{
    api::open_lmdb_env,
    manager::PluginManager,
    stages::{discover::discover_plugins, resolve::resolve_plugins, setup_lua_vm},
};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let env = open_lmdb_env("plugins/steel-data", 16 * 1024 * 1024, 128)?;

    let discovered_plugins = discover_plugins("plugins");
    let resolved_plugins = resolve_plugins(discovered_plugins)?;

    let mut manager = PluginManager::new(setup_lua_vm()?);
    manager.load(resolved_plugins, &env)?;

    manager.enable_all().await?;
    manager.shutdown().await?;

    Ok(())
}
