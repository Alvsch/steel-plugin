use mlua::prelude::*;
use tracing::{debug, error, info, trace, warn};

pub fn install_logger(lua: &Lua, env: &LuaTable, plugin_name: impl Into<String>) -> LuaResult<()> {
    let base_plugin_name = plugin_name.into();
    let plugin_name = base_plugin_name.clone();
    env.set(
        "error",
        lua.create_function(move |_, message: String| {
            error!("[{plugin_name}] {message}");
            Ok(())
        })?,
    )?;

    let plugin_name = base_plugin_name.clone();
    env.set(
        "warn",
        lua.create_function(move |_, message: String| {
            warn!("[{plugin_name}] {message}");
            Ok(())
        })?,
    )?;

    let plugin_name = base_plugin_name.clone();
    env.set(
        "info",
        lua.create_function(move |_, message: String| {
            info!("[{plugin_name}] {message}");
            Ok(())
        })?,
    )?;

    let plugin_name = base_plugin_name.clone();
    env.set(
        "debug",
        lua.create_function(move |_, message: String| {
            debug!("[{plugin_name}] {message}");
            Ok(())
        })?,
    )?;

    let plugin_name = base_plugin_name.clone();
    env.set(
        "trace",
        lua.create_function(move |_, message: String| {
            trace!("[{plugin_name}] {message}");
            Ok(())
        })?,
    )?;

    Ok(())
}
