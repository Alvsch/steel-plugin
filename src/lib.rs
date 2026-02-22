use std::{
    collections::hash_map::Entry,
    path::{Path, PathBuf},
};

pub use logger::Logger;
use mlua::{FromLua, prelude::*};
use rustc_hash::FxHashMap;
pub use signal::{Connection, Signal};
use tokio::fs::{read_dir, read_to_string};

mod logger;
mod signal;

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

fn init_globals(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    globals.set("log", Logger)?;
    Ok(())
}

pub struct PluginLoader {
    lua: Lua,
    plugins: FxHashMap<String, PluginManifest>,
    _data_folder_path: PathBuf,
}

impl PluginLoader {
    pub fn new(data_folder_path: PathBuf) -> LuaResult<Self> {
        let lua = Lua::new();
        init_globals(&lua)?;

        lua.sandbox(true)?;

        Ok(Self {
            lua,
            plugins: FxHashMap::default(),
            _data_folder_path: data_folder_path,
        })
    }

    pub async fn load_all(&mut self, path: &Path) -> LuaResult<()> {
        if path.is_file() {
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

    async fn load_plugin(&mut self, path: &Path) -> LuaResult<()> {
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
