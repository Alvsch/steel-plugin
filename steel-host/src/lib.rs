use mlua::prelude::*;

mod loader;
mod logger;
mod manifest;
mod signal;

pub use loader::PluginLoader;
pub use logger::Logger;
pub use manifest::PluginManifest;
pub use signal::{Connection, Signal};

pub fn init_globals(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    globals.set("log", Logger)?;
    Ok(())
}

pub fn create_env(lua: &Lua) -> anyhow::Result<LuaTable> {
    let env = lua.create_table()?;
    let globals = lua.globals();

    let env_ref = env.clone();

    let mt = lua.create_table()?;
    mt.set(
        "__index",
        lua.create_function(move |_, (_, key): (LuaValue, LuaValue)| {
            if let Ok(v) = env_ref.raw_get::<LuaValue>(key.clone())
                && !matches!(v, LuaValue::Nil)
            {
                return Ok(v);
            }

            globals.get(key)
        })?,
    )?;

    let env_ref = env.clone();
    mt.set(
        "__newindex",
        lua.create_function(move |_, (_, key, val): (LuaValue, LuaValue, LuaValue)| {
            env_ref.raw_set(key, val)
        })?,
    )?;

    env.set_metatable(Some(mt))?;

    Ok(env)
}
