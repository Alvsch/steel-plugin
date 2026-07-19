use std::{collections::HashMap, hash::BuildHasher, sync::Weak};

use mlua::prelude::*;
use steel_utils::locks::SyncRwLock;

use crate::plugin::Plugin;

#[derive(Debug)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl FromLua for Identifier {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let lua_string = value.as_string().ok_or(LuaError::FromLuaConversionError {
            from: "string",
            to: "Namespace".to_string(),
            message: None,
        })?;

        let string = lua_string.to_str()?;
        let (namespace, path) = string
            .split_once(':')
            .ok_or(LuaError::FromLuaConversionError {
                from: "string",
                to: "Namespace".to_string(),
                message: None,
            })?;

        Ok(Identifier {
            namespace: namespace.to_string(),
            path: path.to_string(),
        })
    }
}

pub fn create_require_function<S: BuildHasher + Send + Sync + 'static>(
    lua: &Lua,
    require: Weak<SyncRwLock<HashMap<String, Plugin, S>>>,
) -> LuaResult<LuaFunction> {
    lua.create_function(move |_lua, identifier: Identifier| {
        if let Some(plugins) = require.upgrade() {
            let plugins = plugins.read();
            let Some(value) = plugins.get(&identifier.namespace) else {
                return Ok(LuaValue::Nil);
            };
            let value: LuaValue = value.env.get(identifier.path)?;
            return Ok(value);
        }
        Ok(LuaValue::Nil)
    })
}
