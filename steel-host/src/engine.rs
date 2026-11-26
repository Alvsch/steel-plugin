use std::{
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use tokio::fs::create_dir_all;
use wasmtime::{
    Config, Engine,
    component::{Component, Linker},
};
use wasmtime_wasi::{FsPerms, WasiCtx, WasiCtxBuilder, p2};

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
        p2::add_to_linker_async(&mut linker)?;
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

        let component = Component::new(&self.engine, &bytes).map_err(PluginError::InvalidModule)?;

        Ok(component)
    }

    pub async fn prepare_wasi(&self, plugin_name: &str) -> Result<WasiCtx, PluginError> {
        let plugin_data_folder = self.data_folder.join(plugin_name);
        create_dir_all(&plugin_data_folder)
            .await
            .map_err(PluginError::Io)?;

        let wasi = WasiCtxBuilder::new()
            .preopened_dir(&plugin_data_folder, "/", FsPerms::ReadWrite)
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;
    use wasmtime::{Cache, CacheConfig, Config, component::Component};

    use super::WasmEngine;

    const EMPTY_COMPONENT: &[u8] = b"\0asm\r\0\x01\0";

    fn config_with_cache(data_folder: &Path) -> wasmtime::Result<(Config, Cache)> {
        let mut cache_config = CacheConfig::new();
        let cache_dir = std::path::absolute(data_folder.join(".wasmtime-cache"))?;
        cache_config.with_directory(cache_dir);

        let cache = Cache::new(cache_config)?;
        let mut config = Config::new();
        config.cache(Some(cache.clone()));
        Ok((config, cache))
    }

    #[test]
    fn component_compilation_is_cached_across_engines() -> wasmtime::Result<()> {
        let data_folder = tempdir().map_err(|error| wasmtime::Error::msg(error.to_string()))?;

        {
            let (config, cache) = config_with_cache(data_folder.path())?;
            let wasm_engine = WasmEngine::new(config, data_folder.path().to_path_buf())?;
            Component::new(&wasm_engine.engine, EMPTY_COMPONENT)?;
            assert_eq!(cache.cache_misses(), 1);
            assert_eq!(cache.cache_hits(), 0);
        }

        {
            let (config, cache) = config_with_cache(data_folder.path())?;
            let wasm_engine = WasmEngine::new(config, data_folder.path().to_path_buf())?;
            Component::new(&wasm_engine.engine, EMPTY_COMPONENT)?;
            assert_eq!(cache.cache_misses(), 0);
            assert_eq!(cache.cache_hits(), 1);
        }

        Ok(())
    }

    #[test]
    fn component_compilation_does_not_create_cache_by_default() -> wasmtime::Result<()> {
        let data_folder = tempdir().map_err(|error| wasmtime::Error::msg(error.to_string()))?;
        let wasm_engine = WasmEngine::new(Config::new(), data_folder.path().to_path_buf())?;
        Component::new(&wasm_engine.engine, EMPTY_COMPONENT)?;

        assert!(!data_folder.path().join(".wasmtime-cache").exists());
        Ok(())
    }
}
