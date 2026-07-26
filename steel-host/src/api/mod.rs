use heed::Env;
use mlua::prelude::*;

mod data_store;
mod logging;
mod signal;

pub use data_store::{DataStore, LmdbStore, MemoryStore, open_lmdb_env};
pub use logging::install_logger;
pub use signal::{Connection, Signal};

use crate::require::{install_require, registry::PluginRegistry};

#[expect(clippy::unnecessary_wraps, clippy::missing_const_for_fn)]
pub(crate) fn install_base_globals(_lua: &Lua) -> mlua::Result<()> {
    Ok(())
}

pub(crate) fn install_plugin_globals(
    lua: &Lua,
    env: &LuaTable,
    registry: PluginRegistry,
    plugin_name: &str,
    database_env: Env,
) -> mlua::Result<()> {
    install_require(lua, env, registry, plugin_name)?;
    install_logger(lua, env, plugin_name)?;

    let store = LmdbStore::open(database_env, plugin_name)?;
    env.set("lmdb", store)?;

    Ok(())
}
