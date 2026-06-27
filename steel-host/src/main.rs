use steel_host::{PluginLoader, Signal};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let signal: Signal<String> = Signal::new();

    let mut loader = PluginLoader::new("plugins", |globals| {
        globals.set("signal", signal.clone())?;
        Ok(())
    })?;

    loader.load_all("examples").await?;

    signal.emit("wow");

    loader.unload_all().await?;
    Ok(())
}
