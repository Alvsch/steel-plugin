pub mod compile;
pub mod discover;
pub mod execute;
pub mod resolve;

use mlua::{Lua, LuaOptions, StdLib};
use thiserror::Error;

use crate::api::install_base_globals;

#[derive(Debug, Error)]
pub enum VmInitError {
    #[error("failed to create Lua VM: {source}")]
    Create {
        #[source]
        source: mlua::Error,
    },

    #[error("failed to configure sandbox: {source}")]
    Sandbox {
        #[source]
        source: mlua::Error,
    },

    #[error("failed to install base globals: {source}")]
    Globals {
        #[source]
        source: mlua::Error,
    },
}

pub fn setup_lua_vm() -> Result<Lua, VmInitError> {
    let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())
        .map_err(|source| VmInitError::Create { source })?;

    install_base_globals(&lua).map_err(|source| VmInitError::Globals { source })?;

    lua.sandbox(true)
        .map_err(|source| VmInitError::Sandbox { source })?;
    lua.globals().set_readonly(true);

    Ok(lua)
}
