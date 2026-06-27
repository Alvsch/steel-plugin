use mlua::prelude::*;
use semver::Version;

#[derive(Debug)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: Version,
    pub api_version: Version,
    pub on_enable: LuaFunction,
    pub on_disable: LuaFunction,
}

impl FromLua for PluginManifest {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        Ok(PluginManifest {
            name: table.get("name")?,
            description: table.get("description")?,
            author: table.get("author")?,
            version: Version::parse(&table.get::<String>("version")?).map_err(|err| {
                LuaError::FromLuaConversionError {
                    from: "string",
                    to: "Version".to_string(),
                    message: Some(err.to_string()),
                }
            })?,
            api_version: Version::parse(&table.get::<String>("api_version")?).map_err(|err| {
                LuaError::FromLuaConversionError {
                    from: "string",
                    to: "Version".to_string(),
                    message: Some(err.to_string()),
                }
            })?,
            on_enable: table.get("on_enable")?,
            on_disable: table.get("on_disable")?,
        })
    }
}
