use std::{
    collections::{HashMap, hash_map::Entry},
    path::{Path, PathBuf},
};

use mlua::prelude::*;
use tokio::fs::{read_dir, read_to_string};

use crate::{init_globals, manifest::PluginManifest};

pub struct PluginLoader {
    lua: Lua,
    plugins: HashMap<String, PluginManifest>,
    _data_folder_path: PathBuf,
}

impl PluginLoader {
    pub fn new(data_folder_path: PathBuf) -> LuaResult<Self> {
        let lua = Lua::new();
        init_globals(&lua)?;

        lua.sandbox(true)?;
        lua.globals().set_readonly(true);

        Ok(Self {
            lua,
            plugins: HashMap::default(),
            _data_folder_path: data_folder_path,
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
        let source = read_to_string(path).await?;
        let chunk = self.lua.load(source);

        let manifest: PluginManifest = chunk.eval()?;
        let manifest = match self.plugins.entry(manifest.name.clone()) {
            Entry::Occupied(entry) => panic!("plugin with name {:?} already exists", entry.key()),
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
