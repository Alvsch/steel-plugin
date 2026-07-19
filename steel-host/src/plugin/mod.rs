use mlua::prelude::*;

pub use manifest::PluginManifest;

mod manifest;

#[derive(Debug)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub env: LuaTable,
}
