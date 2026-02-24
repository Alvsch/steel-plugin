use std::path::{Path, PathBuf};

use serde::Deserialize;
use tokio::fs::{create_dir_all, read};
use wasmparser::{Parser, Payload};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder, p1::WasiP1Ctx};

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
    pub fn new(engine: Engine, linker: Linker<PluginHostData>, data_dir: PathBuf) -> Self {
        Self {
            engine,
            linker,
            data_dir,
        }
    }

    pub async fn load_plugin(&self, path: &Path) {
        let bytes = read(path).await.unwrap();

        // manifest
        let meta = read_custom_section(&bytes, "plugin_meta").unwrap().unwrap();
        let meta: PluginMeta = rmp_serde::from_slice(meta).unwrap();
        println!("{:#?}", meta);

        // compile
        let precompiled = self.engine.precompile_module(&bytes).unwrap();
        let module = unsafe { Module::deserialize(&self.engine, precompiled) }.unwrap();

        // init
        let host_path = self.data_dir.join(meta.name);
        create_dir_all(&host_path).await.unwrap();
        let wasi = WasiCtxBuilder::new()
            .preopened_dir(
                host_path,
                "/data",
                DirPerms::all(),
                FilePerms::all(),
            )
            .unwrap()
            .build_p1();

        let mut store = Store::new(&self.engine, PluginHostData {
            wasi,
            plugin_name: meta.name.to_string(),
        });

        let instance = self
            .linker
            .instantiate_async(&mut store, &module)
            .await
            .unwrap();

        let alloc_fn = instance
            .get_typed_func::<u32, u32>(&mut store, "alloc")
            .unwrap();
        let dealloc_fn = instance
            .get_typed_func::<(u32, u32), ()>(&mut store, "dealloc")
            .unwrap();
        let on_load = instance
            .get_typed_func::<(u32, u32), ()>(&mut store, "on_load")
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // alloc
        let bytes = rmp_serde::to_vec("Steve").unwrap();
        let len = bytes.len() as u32;
        let ptr = alloc_fn.call_async(&mut store, len).await.unwrap();
        memory.write(&mut store, ptr as usize, &bytes).unwrap();

        // load
        on_load.call_async(&mut store, (ptr, len)).await.unwrap();

        // dealloc
        dealloc_fn.call_async(&mut store, (ptr, len)).await.unwrap();
    }
}
