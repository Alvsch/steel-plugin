use std::path::{Path, PathBuf};

use serde::Deserialize;
use tokio::fs::{create_dir_all, read};
use wasmparser::{Parser, Payload};
use wasmtime::{Engine, Instance, Linker, Module, Store, TypedFunc};
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

pub struct PluginExports {
    pub alloc: TypedFunc<u32, u32>,
    pub dealloc: TypedFunc<(u32, u32), ()>,
    pub on_load: TypedFunc<(u32, u32), ()>,
    pub on_unload: TypedFunc<(), ()>,
}

impl PluginExports {
    pub fn resolve(instance: &Instance, store: &mut Store<PluginHostData>) -> anyhow::Result<Self> {
        Ok(Self {
            alloc: instance.get_typed_func(&mut *store, "alloc")?,
            dealloc: instance.get_typed_func(&mut *store, "dealloc")?,
            on_load: instance.get_typed_func(&mut *store, "on_load")?,
            on_unload: instance.get_typed_func(&mut *store, "on_unload")?,
        })
    }
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

    pub async fn load_plugin(&self, path: &Path) {
        let bytes = read(path).await.unwrap();

        // manifest
        let meta = read_custom_section(&bytes, "plugin_meta").unwrap().unwrap();
        let meta: PluginMeta = rmp_serde::from_slice(meta).unwrap();
        println!("{meta:#?}");

        // compile
        let precompiled = self.engine.precompile_module(&bytes).unwrap();
        let module = unsafe { Module::deserialize(&self.engine, precompiled) }.unwrap();

        // init
        let host_path = self.data_dir.join(meta.name);
        create_dir_all(&host_path).await.unwrap();
        let wasi = WasiCtxBuilder::new()
            .preopened_dir(host_path, "/data", DirPerms::all(), FilePerms::all())
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
        let memory = instance.get_memory(&mut store, "memory").unwrap();

        // alloc
        let bytes = b"Steve";
        let len = bytes.len() as u32;
        let ptr = exports.alloc.call_async(&mut store, len).await.unwrap();
        memory.write(&mut store, ptr as usize, bytes).unwrap();

        // load
        exports
            .on_load
            .call_async(&mut store, (ptr, len))
            .await
            .unwrap();

        // dealloc
        exports
            .dealloc
            .call_async(&mut store, (ptr, len))
            .await
            .unwrap();
    }
}
