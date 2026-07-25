use std::{collections::HashMap, fs, io, path::PathBuf};

use mlua::prelude::*;
use thiserror::Error;

use crate::{
    require::registry::PluginRegistry,
    stages::{
        compile::CompiledPlugin,
        execute::{ExecuteError, create_plugin_env},
    },
};

pub mod registry;

enum ModuleState {
    InProgress,
    Done(LuaRegistryKey),
}

pub struct PluginRuntime {
    pub name: String,
    pub env: LuaRegistryKey,
    pub file_table: HashMap<String, PathBuf>,
    pub exports: HashMap<String, String>,
    internal_modules: HashMap<String, ModuleState>,
}

impl PluginRuntime {
    pub fn from_compiled(lua: &Lua, plugin: &CompiledPlugin) -> Result<Self, ExecuteError> {
        let env = create_plugin_env(lua).map_err(|source| ExecuteError::EnvInit {
            plugin: plugin.config.name.clone(),
            source,
        })?;

        let env = lua
            .create_registry_value(env)
            .map_err(|source| ExecuteError::EnvInit {
                plugin: plugin.config.name.clone(),
                source,
            })?;

        Ok(Self {
            name: plugin.config.name.clone(),
            file_table: plugin.file_table.clone(),
            exports: plugin.config.exports.clone(),
            env,
            internal_modules: HashMap::new(),
        })
    }

    pub fn cleanup(self, lua: &Lua) -> mlua::Result<()> {
        lua.remove_registry_value(self.env)?;

        for (_, state) in self.internal_modules {
            if let ModuleState::Done(key) = state {
                lua.remove_registry_value(key)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RequireError {
    #[error("unknown plugin '{0}'")]
    UnknownPlugin(String),
    #[error("module '{0}' not found in plugin '{1}'")]
    ModuleNotFound(String, String),
    #[error("plugin '{0}' has no export '{1}'")]
    ExportNotFound(String, String),
    #[error("export '{key}' in plugin '{plugin}' points to missing file '{path}'")]
    ExportPathMissing {
        plugin: String,
        key: String,
        path: String,
    },
    #[error("circular require: '{0}' in plugin '{1}'")]
    CircularRequire(String, String),
    #[error("invalid require path '{0}', expected './module' or '@plugin/key'")]
    InvalidPath(String),
    #[error("failed to read '{path}': {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("lua error in '{module}': {source}")]
    Lua { module: String, source: mlua::Error },
}

impl From<RequireError> for mlua::Error {
    fn from(e: RequireError) -> Self {
        mlua::Error::RuntimeError(e.to_string())
    }
}

pub fn install_require(
    lua: &Lua,
    env: &LuaTable,
    registry: PluginRegistry,
    plugin_name: String,
) -> mlua::Result<()> {
    let f = lua.create_function(move |lua, path: String| {
        resolve(lua, &registry, &plugin_name, &path).map_err(mlua::Error::from)
    })?;
    env.set("require", f)
}

fn resolve(
    lua: &Lua,
    registry: &PluginRegistry,
    plugin_name: &str,
    path: &str,
) -> Result<LuaValue, RequireError> {
    if let Some(rest) = path.strip_prefix("./") {
        require_internal(lua, registry, plugin_name, rest)
    } else if let Some(rest) = path.strip_prefix('@') {
        let (target, key) = rest
            .split_once('/')
            .ok_or_else(|| RequireError::InvalidPath(path.to_string()))?;
        require_external(lua, registry, target, key)
    } else {
        Err(RequireError::InvalidPath(path.to_string()))
    }
}

fn normalize(path: &str) -> String {
    let path = path.trim_start_matches("./");

    path.strip_suffix(".luau")
        .or_else(|| path.strip_suffix(".lua"))
        .or_else(|| path.strip_suffix(".luac"))
        .unwrap_or(path)
        .to_string()
}

fn plugin_env(
    lua: &Lua,
    registry: &PluginRegistry,
    plugin_name: &str,
) -> Result<LuaTable, RequireError> {
    let reg = registry.lock();
    let plugin = reg
        .get(plugin_name)
        .ok_or_else(|| RequireError::UnknownPlugin(plugin_name.to_string()))?;
    lua.registry_value(&plugin.env)
        .map_err(|source| RequireError::Lua {
            module: plugin_name.to_string(),
            source,
        })
}

/// require("./module") — cached, cycle-checked.
fn require_internal(
    lua: &Lua,
    registry: &PluginRegistry,
    plugin_name: &str,
    module_path: &str,
) -> Result<LuaValue, RequireError> {
    let norm = normalize(module_path);

    // 1. Cache / cycle check. Borrow ends before any Lua call below.
    {
        let reg = registry.lock();
        let plugin = reg
            .get(plugin_name)
            .ok_or_else(|| RequireError::UnknownPlugin(plugin_name.to_string()))?;
        match plugin.internal_modules.get(&norm) {
            Some(ModuleState::Done(key)) => {
                return lua.registry_value(key).map_err(|source| RequireError::Lua {
                    module: norm.clone(),
                    source,
                });
            }
            Some(ModuleState::InProgress) => {
                return Err(RequireError::CircularRequire(norm, plugin_name.to_string()));
            }
            None => (),
        }
    }

    // 2. Resolve disk path, mark in-progress. Borrow ends before eval.
    let disk_path = {
        let mut reg = registry.lock();
        let plugin = reg
            .get_mut(plugin_name)
            .ok_or_else(|| RequireError::UnknownPlugin(plugin_name.to_string()))?;
        let disk_path =
            plugin.file_table.get(&norm).cloned().ok_or_else(|| {
                RequireError::ModuleNotFound(norm.clone(), plugin_name.to_string())
            })?;
        plugin
            .internal_modules
            .insert(norm.clone(), ModuleState::InProgress);
        disk_path
    };

    let env = plugin_env(lua, registry, plugin_name)?;

    // 3. Execute. NO registry borrow held here — a nested "./" require inside
    // this chunk safely re-enters steps 1-2.
    let exec_result = if disk_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("luac"))
    {
        let src = fs::read(&disk_path).map_err(|source| RequireError::Io {
            path: disk_path.clone(),
            source,
        })?;
        lua.load(&src)
            .set_name(format!("={plugin_name}/{norm}"))
            .set_environment(env)
            .eval::<LuaValue>()
    } else {
        let src = fs::read_to_string(&disk_path).map_err(|source| RequireError::Io {
            path: disk_path.clone(),
            source,
        })?;
        lua.load(&src)
            .set_name(format!("={plugin_name}/{norm}"))
            .set_environment(env)
            .eval::<LuaValue>()
    };

    // 4. Commit result, or clear the InProgress ghost on failure so a later
    // (non-circular) require of the same module isn't permanently poisoned.
    let mut reg = registry.lock();
    let plugin = reg
        .get_mut(plugin_name)
        .ok_or_else(|| RequireError::UnknownPlugin(plugin_name.to_string()))?;
    match exec_result {
        Ok(value) => {
            let key =
                lua.create_registry_value(value.clone())
                    .map_err(|source| RequireError::Lua {
                        module: norm.clone(),
                        source,
                    })?;
            plugin.internal_modules.insert(norm, ModuleState::Done(key));
            Ok(value)
        }
        Err(source) => {
            plugin.internal_modules.remove(&norm);
            Err(RequireError::Lua {
                module: norm,
                source,
            })
        }
    }
}

/// require("@target/key") — no cache, executes in the REQUESTER's env.
/// `target` must already be in the registry (guaranteed post-Execute for all
/// declared dependencies).
fn require_external(
    lua: &Lua,
    registry: &PluginRegistry,
    target_name: &str,
    export_key: &str,
) -> Result<LuaValue, RequireError> {
    let disk_path =
        {
            let reg = registry.lock();
            let target = reg
                .get(target_name)
                .ok_or_else(|| RequireError::UnknownPlugin(target_name.to_string()))?;
            let module_path = target.exports.get(export_key).ok_or_else(|| {
                RequireError::ExportNotFound(target_name.to_string(), export_key.to_string())
            })?;
            let norm = normalize(module_path);
            target.file_table.get(&norm).cloned().ok_or_else(|| {
                RequireError::ExportPathMissing {
                    plugin: target_name.to_string(),
                    key: export_key.to_string(),
                    path: module_path.clone(),
                }
            })?
        };

    let env = plugin_env(lua, registry, target_name)?;
    let chunk_name = format!("=@{target_name}/{export_key}");

    let result = if disk_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("luac"))
    {
        let src = fs::read(&disk_path).map_err(|source| RequireError::Io {
            path: disk_path.clone(),
            source,
        })?;
        lua.load(&src)
            .set_name(&chunk_name)
            .set_environment(env)
            .eval::<LuaValue>()
    } else {
        let src = fs::read_to_string(&disk_path).map_err(|source| RequireError::Io {
            path: disk_path.clone(),
            source,
        })?;
        lua.load(&src)
            .set_name(&chunk_name)
            .set_environment(env)
            .eval::<LuaValue>()
    };

    result.map_err(|source| RequireError::Lua {
        module: format!("@{target_name}/{export_key}"),
        source,
    })
}
