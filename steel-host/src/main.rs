use mlua::prelude::*;
use steel_host::{
    PluginLoader,
    api::{MemoryStore, Signal},
};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let signal: Signal<String> = Signal::new();

    let loader = PluginLoader::new("tests", |lua| {
        let globals = lua.globals();

        let game = lua.create_table()?;
        game.set("Store", MemoryStore::new())?;
        globals.set("game", game)?;

        globals.set("signal", signal.clone())?;
        globals.set("Signal", lua.create_proxy::<Signal<LuaValue>>()?)?;
        Ok(())
    })?;

    loader.load_all("plugins").await?;

    signal.emit("wow".to_string());

    loader.unload_all().await?;
    Ok(())
}
