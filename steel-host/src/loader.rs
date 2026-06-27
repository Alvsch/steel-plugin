use std::{
    collections::{HashMap, hash_map::Entry},
    path::{Path, PathBuf},
};

use mlua::prelude::*;
use tokio::fs::{read_dir, read_to_string};

use crate::{create_env, init_globals, manifest::PluginManifest};

pub struct PluginLoader {
    pub lua: Lua,
    plugins: HashMap<String, PluginManifest>,
    _data_folder_path: PathBuf,
}

impl PluginLoader {
    pub fn new(
        data_folder_path: impl Into<PathBuf>,
        register_globals: impl Fn(&LuaTable) -> LuaResult<()>,
    ) -> LuaResult<Self> {
        let lua = Lua::new();

        let globals = lua.globals();
        init_globals(&lua, &globals)?;

        (register_globals)(&globals)?;

        lua.sandbox(true)?;
        lua.globals().set_readonly(true);

        Ok(Self {
            lua,
            plugins: HashMap::default(),
            _data_folder_path: data_folder_path.into(),
        })
    }

    pub async fn load_all(&mut self, path: impl AsRef<Path>) -> LuaResult<()> {
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

    async fn load_plugin(&mut self, path: impl AsRef<Path>) -> LuaResult<()> {
        let path = path.as_ref();
        let source = read_to_string(path).await?;

        let env = create_env(&self.lua)?;
        let chunk = self
            .lua
            .load(source)
            .set_environment(env)
            .set_name(path.display().to_string());

        let manifest: PluginManifest = chunk.eval()?;
        let manifest = match self.plugins.entry(manifest.name.clone()) {
            Entry::Occupied(entry) => panic!("plugin \"{:?}\" already exists", entry.key()),
            Entry::Vacant(entry) => entry.insert(manifest),
        };

        manifest.on_enable.call_async::<()>(()).await?;
        Ok(())
    }

    pub async fn unload_all(&mut self) -> LuaResult<()> {
        for (_name, manifest) in self.plugins.drain() {
            manifest.on_disable.call_async::<()>(()).await?;
        }
        Ok(())
    }
}
