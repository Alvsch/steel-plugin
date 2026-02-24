use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use tokio::fs::{create_dir_all, read, read_dir};
use tracing::error;
use wasmparser::{Parser, Payload};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder, p1::WasiP1Ctx};

use crate::{container::PluginContainer, exports::PluginExports};

mod container;
pub mod exports;
mod topological_sort;

#[derive(Debug, Deserialize)]
pub struct PluginMeta<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub api_version: u32,
    pub depends: Vec<&'a str>,
}

pub struct PluginHostData {
    pub wasi: WasiP1Ctx,
    pub plugin_name: String,
}

fn read_custom_section<'a>(bytes: &'a [u8], name: &str) -> anyhow::Result<Option<&'a [u8]>> {
    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::CustomSection(reader) if reader.name() == name => {
                return Ok(Some(reader.data()));
            }
            _ => {}
        }
    }
    Ok(None)
}

pub struct PluginLoader {
    engine: Engine,
    linker: Linker<PluginHostData>,
    data_dir: PathBuf,
}

impl PluginLoader {
    #[must_use]
    pub const fn new(engine: Engine, linker: Linker<PluginHostData>, data_dir: PathBuf) -> Self {
        Self {
            engine,
            linker,
            data_dir,
        }
    }

    pub async fn discover_plugins(&self, plugin_dir: &Path) -> Vec<PluginContainer> {
        let mut plugins = Vec::new();

        let mut dir = read_dir(plugin_dir).await.unwrap();
        while let Some(entry) = dir.next_entry().await.unwrap() {
            let file_type = entry.file_type().await.unwrap();
            let file_path = entry.path();

            if file_type.is_file() {
                if file_path
                    .extension()
                    .is_none_or(|x| x.to_str().is_none_or(|ext| ext != "wasm"))
                {
                    continue;
                }
                let bytes = read(&file_path).await.unwrap();
                let container = PluginContainer::new(bytes, |bytes| {
                    let meta = read_custom_section(bytes, "plugin_meta").unwrap().unwrap();
                    let meta: PluginMeta = rmp_serde::from_slice(meta).unwrap();
                    meta
                });
                plugins.push(container);
            }
        }
        let (topology, invalid) = topological_sort::sort_plugins(plugins);
        if !invalid.is_empty() {
            error!("invalid: {:#?}", invalid);
        }

        topology
    }

    pub async fn load_plugin(&self, container: PluginContainer) {
        let bytes = container.borrow_owner();
        let meta = container.borrow_dependent();

        println!("{:#?}", meta);

        // compile
        let precompiled = self.engine.precompile_module(bytes).unwrap();
        let module = unsafe { Module::deserialize(&self.engine, precompiled) }.unwrap();

        // init
        let host_path = self.data_dir.join(meta.name);
        create_dir_all(&host_path).await.unwrap();
        let wasi = WasiCtxBuilder::new()
            .preopened_dir(host_path, "/", DirPerms::all(), FilePerms::all())
            .unwrap()
            .build_p1();

        let mut store = Store::new(
            &self.engine,
            PluginHostData {
                wasi,
                plugin_name: meta.name.to_string(),
            },
        );

        let instance = self
            .linker
            .instantiate_async(&mut store, &module)
            .await
            .unwrap();

        let exports = PluginExports::resolve(&instance, &mut store).unwrap();

        // load
        exports.on_load.call_async(&mut store, ()).await.unwrap();
    }
}
