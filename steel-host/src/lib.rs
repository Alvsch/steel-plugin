use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf};
use tracing::info;
use wasmparser::{Parser, Payload};
use wasmtime::{Caller, Linker};
use wasmtime_wasi::p1::wasi_snapshot_preview1::add_to_linker;

pub use wasmtime;

pub use crate::event::EventRegistry;
pub use crate::exports::PluginExports;
pub use crate::loader::{PluginHostData, PluginLoader, PluginLoaderError};
pub use crate::manager::PluginManager;
pub use instance::PluginInstance;

mod event;
mod exports;
mod instance;
mod loader;
mod manager;
mod topological_sort;

fn read_custom_section<'a>(bytes: &'a [u8], name: &str) -> wasmparser::Result<Option<&'a [u8]>> {
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

#[derive(Debug, Deserialize, Clone)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub depends: Box<[String]>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

pub fn configure_linker(linker: &mut Linker<PluginHostData>) {
    add_to_linker(linker, |data: &mut PluginHostData| &mut data.wasi).unwrap();
    linker
        .func_wrap(
            "host",
            "info",
            |mut caller: Caller<PluginHostData>, ptr: u32, len: u32| {
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let buf = &memory.data(&caller)[ptr as usize..ptr as usize + len as usize];
                let message = str::from_utf8(buf).unwrap();

                let plugin_name = caller.data().name.as_str();
                info!("[{plugin_name}] {message}");
            },
        )
        .unwrap();
    linker
        .func_wrap_async(
            "host",
            "register_handler",
            |mut caller: Caller<PluginHostData>, (ptr, len): (u32, u32)| {
                Box::new(async move {
                    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                    let data = &memory.data(&caller)[ptr as usize..(ptr + len) as usize];
                    let handler = rmp_serde::from_slice(data).unwrap();

                    let registry = &*caller.data().registry;
                    let plugin_name = caller.data().name.clone();
                    registry.register_handler(plugin_name, handler).await;
                })
            },
        )
        .unwrap();
}
