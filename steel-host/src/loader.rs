use std::{
    collections::{HashMap, hash_map::Entry},
    path::{Path, PathBuf},
    sync::Arc,
};

use mlua::prelude::*;
use steel_utils::locks::SyncRwLock;
use tokio::fs::{read_dir, read_to_string};

use crate::{
    api::create_require_function,
    create_env, init_globals,
    plugin::{Plugin, PluginManifest},
};

pub struct PluginLoader {
    pub lua: Lua,
    plugins: Arc<SyncRwLock<HashMap<String, Plugin>>>,
    _data_folder_path: PathBuf,
}

impl PluginLoader {
    pub fn new(
        data_folder_path: impl Into<PathBuf>,
        register_globals: impl Fn(&Lua) -> LuaResult<()>,
    ) -> LuaResult<Self> {
        let lua = Lua::new();
        init_globals(&lua)?;

        let plugins = Arc::new(SyncRwLock::new(HashMap::new()));
        lua.globals().set(
            "require",
            create_require_function(&lua, Arc::downgrade(&plugins))?,
        )?;

        (register_globals)(&lua)?;

        lua.sandbox(true)?;
        lua.globals().set_readonly(true);

        Ok(Self {
            lua,
            plugins,
            _data_folder_path: data_folder_path.into(),
        })
    }

    pub async fn load_all(&self, path: impl AsRef<Path>) -> LuaResult<()> {
        if path.as_ref().is_file() {
            return self.load_plugin(path).await;
        }

        let mut read = read_dir(path).await?;
        while let Ok(Some(entry)) = read.next_entry().await {
            let path = entry.path();
            if !matches!(
                path.extension().and_then(|x| x.to_str()),
                Some("lua" | "luau")
            ) {
                continue;
            }
            self.load_plugin(&path).await?;
        }

        Ok(())
    }

    async fn load_plugin(&self, path: impl AsRef<Path>) -> LuaResult<()> {
        let path = path.as_ref();
        let source = read_to_string(path).await?;

        let env = create_env(&self.lua)?;
        let chunk = self
            .lua
            .load(source)
            .set_environment(env.clone())
            .set_name(path.display().to_string());

        let manifest: PluginManifest = chunk.eval()?;
        let on_enable = {
            let mut write = self.plugins.write();
            let plugin = match write.entry(manifest.name.clone()) {
                Entry::Occupied(entry) => panic!("plugin \"{:?}\" already exists", entry.key()),
                Entry::Vacant(entry) => entry.insert(Plugin { manifest, env }),
            };
            plugin.manifest.on_enable.clone()
        };

        on_enable.call_async::<()>(()).await?;
        Ok(())
    }

    pub async fn unload_all(&self) -> LuaResult<()> {
        let plugins = self
            .plugins
            .write()
            .drain()
            .map(|(_, plugin)| plugin.manifest.on_disable)
            .collect::<Vec<_>>();

        for on_disable in plugins {
            on_disable.call_async::<()>(()).await?;
        }
        Ok(())
    }
}
