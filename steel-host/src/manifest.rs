use mlua::prelude::*;

#[derive(Debug)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub on_enable: LuaFunction,
    pub on_disable: LuaFunction,
}

impl FromLua for PluginManifest {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        Ok(PluginManifest {
            name: table.get("name")?,
            description: table.get("description")?,
            version: table.get("version")?,
            author: table.get("author")?,
            on_enable: table.get("on_enable")?,
            on_disable: table.get("on_disable")?,
        })
    }
}
