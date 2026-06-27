use mlua::prelude::*;
use steel_host::{PluginManifest, Signal, create_env, init_globals};
use tracing::Level;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let plugin = include_str!("../../examples/signals.lua");

    let lua = Lua::new();

    // setup globals
    init_globals(&lua)?;

    let signal: Signal<String> = Signal::new();
    lua.globals().set("signal", signal.clone())?;

    // setup sandbox
    lua.sandbox(true)?;
    lua.globals().set_readonly(true);

    // load plugin
    let plugin_env = create_env(&lua)?;

    let plugin = lua
        .load(plugin)
        .set_name("plugin")
        .set_environment(plugin_env)
        .eval::<PluginManifest>()?;

    // enable plugin
    plugin.on_enable.call::<()>(())?;
    signal.emit("wow");

    // disable plugin
    plugin.on_disable.call::<()>(())?;

    Ok(())
}
