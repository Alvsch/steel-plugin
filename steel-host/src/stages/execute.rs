use mlua::prelude::*;
use thiserror::Error;

use crate::{
    require::{PluginRuntime, install_require, registry::PluginRegistry},
    stages::compile::CompiledPlugin,
};

pub struct LoadedPlugin {
    pub name: String,
    on_enable: LuaRegistryKey,
    on_disable: LuaRegistryKey,
}

impl LoadedPlugin {
    pub fn on_enable(&self, lua: &Lua) -> mlua::Result<()> {
        let on_enable = lua.registry_value::<LuaFunction>(&self.on_enable)?;
        on_enable.call::<()>(())?;
        Ok(())
    }

    pub fn on_disable(&self, lua: &Lua) -> mlua::Result<()> {
        let on_disable = lua.registry_value::<LuaFunction>(&self.on_disable)?;
        on_disable.call::<()>(())?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("failed to create env for plugin '{plugin}': {source}")]
    EnvInit { plugin: String, source: mlua::Error },

    #[error("plugin '{plugin}' failed during init execution: {source}")]
    Exec { plugin: String, source: mlua::Error },

    #[error("plugin '{plugin}' init.lua did not return a table")]
    NotATable { plugin: String },

    #[error("plugin '{plugin}' return table missing '{field}' function")]
    MissingField { plugin: String, field: &'static str },
}

pub fn create_plugin_env(lua: &Lua) -> mlua::Result<LuaTable> {
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

pub fn execute_plugin(
    lua: &Lua,
    registry: PluginRegistry,
    plugin: CompiledPlugin,
) -> Result<LoadedPlugin, ExecuteError> {
    let runtime = PluginRuntime::from_compiled(lua, &plugin)?;
    let name = runtime.name.clone();

    let env = lua
        .registry_value(&runtime.env)
        .map_err(|source| ExecuteError::EnvInit {
            plugin: name.clone(),
            source,
        })?;

    registry.lock().insert(name.clone(), runtime);

    install_require(lua, &env, registry, name.clone()).map_err(|source| ExecuteError::EnvInit {
        plugin: name.clone(),
        source,
    })?;

    // bind host API + custom require directly into `env` (not globals),
    // scoping visibility to this plugin only
    bind_host_api(lua, &env, &plugin).map_err(|source| ExecuteError::EnvInit {
        plugin: name.clone(),
        source,
    })?;

    let result: LuaValue = lua
        .load(&plugin.init_bytecode)
        .set_name(&plugin.config.name)
        .set_environment(env.clone())
        .eval()
        .map_err(|source| ExecuteError::Exec {
            plugin: name.clone(),
            source,
        })?;

    let LuaValue::Table(table) = result else {
        return Err(ExecuteError::NotATable {
            plugin: name.clone(),
        });
    };

    let on_enable = get_required_fn(lua, &table, "on_enable", &name)?;
    let on_disable = get_required_fn(lua, &table, "on_disable", &name)?;

    Ok(LoadedPlugin {
        name,
        on_enable,
        on_disable,
    })
}

fn get_required_fn(
    lua: &Lua,
    table: &LuaTable,
    field: &'static str,
    plugin_name: &str,
) -> Result<LuaRegistryKey, ExecuteError> {
    let value: LuaValue = table.get(field).map_err(|source| ExecuteError::Exec {
        plugin: plugin_name.to_string(),
        source,
    })?;

    let LuaValue::Function(func) = value else {
        return Err(ExecuteError::MissingField {
            plugin: plugin_name.to_string(),
            field,
        });
    };

    lua.create_registry_value(func)
        .map_err(|source| ExecuteError::Exec {
            plugin: plugin_name.to_string(),
            source,
        })
}

const fn bind_host_api(_lua: &Lua, _env: &LuaTable, _plugin: &CompiledPlugin) -> mlua::Result<()> {
    // TODO: require (./ via file_table, @name/ via host RPC), Signal, Scheduler,
    // UserData host bindings — all raw_set into `env`
    Ok(())
}
