use steel_host::{
    PluginLoader,
    api::{MemoryStore, Signal},
};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let signal: Signal<&str> = Signal::new();

    let mut loader = PluginLoader::new("tests", |lua| {
        let globals = lua.globals();

        let game = lua.create_table()?;
        game.set("Store", MemoryStore::new())?;
        globals.set("game", game)?;

        globals.set("signal", signal.clone())?;
        Ok(())
    })?;

    loader.load_all("examples").await?;

    signal.emit("wow");

    loader.unload_all().await?;
    Ok(())
}
