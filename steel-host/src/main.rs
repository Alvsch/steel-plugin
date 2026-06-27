use steel_host::{
    PluginLoader,
    api::{data_store::DataStore, signal::Signal},
};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let signal: Signal<String> = Signal::new();

    let mut loader = PluginLoader::new("plugins", |lua| {
        let globals = lua.globals();

        let game = lua.create_table()?;
        let store = DataStore::new();
        game.set("Store", store)?;
        globals.set("game", game)?;

        globals.set("signal", signal.clone())?;
        Ok(())
    })?;

    loader.load_all("examples").await?;

    signal.emit("wow");

    loader.unload_all().await?;
    Ok(())
}
