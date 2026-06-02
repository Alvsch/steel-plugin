use std::{
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use tokio::fs::create_dir_all;
use wasmtime::{
    Config, Engine,
    component::{Component, Linker},
};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, p2::add_to_linker_async};

use crate::{
    error::PluginError,
    linker::{self, HostLinker},
};

pub struct WasmEngine {
    pub engine: Engine,
    pub linker: HostLinker,
    pub data_folder: PathBuf,
}

impl WasmEngine {
    pub fn new(config: Config, data_folder: PathBuf) -> wasmtime::Result<Self> {
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        add_to_linker_async(&mut linker)?;
        linker::add_to_linker(&mut linker)?;
        Ok(Self {
            engine,
            linker,
            data_folder,
        })
    }

    pub async fn compile_plugin(&self, file_path: &Path) -> Result<Component, PluginError> {
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|err| match err.kind() {
                ErrorKind::NotFound => PluginError::NotFound {
                    file_path: file_path.to_path_buf(),
                },
                _ => PluginError::Io(err),
            })?;

        let precompiled = self
            .engine
            .precompile_component(&bytes)
            .map_err(PluginError::InvalidModule)?;

        let component = unsafe { Component::deserialize(&self.engine, precompiled) }
            .map_err(PluginError::InvalidModule)?;

        Ok(component)
    }

    pub async fn prepare_wasi(&self, plugin_name: &str) -> Result<WasiCtx, PluginError> {
        let plugin_data_folder = self.data_folder.join(plugin_name);
        create_dir_all(&plugin_data_folder)
            .await
            .map_err(PluginError::Io)?;

        let wasi = WasiCtxBuilder::new()
            .preopened_dir(&plugin_data_folder, "/", DirPerms::all(), FilePerms::all())
            .map_err(|err| {
                PluginError::Io(
                    err.downcast::<io::Error>()
                        .expect(".preopened_dir() can only return an io::Error"),
                )
            })?
            .build();

        Ok(wasi)
    }
}
