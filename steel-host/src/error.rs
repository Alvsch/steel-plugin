use rmp_serde::decode;
use std::io;
use thiserror::Error;
use wasmparser::BinaryReaderError;

#[derive(Debug, Error)]
pub enum PluginLoaderError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("binary reader: {0}")]
    BinaryReader(#[from] BinaryReaderError),
    #[error("missing plugin meta")]
    MissingPluginMeta,
    #[error("invalid plugin meta: {0}")]
    InvalidPluginMeta(decode::Error),
    #[error("wasmtime: {0}")]
    Wasmtime(#[from] wasmtime::Error),
    #[error("plugin export resolve: {0}")]
    PluginExportResolve(wasmtime::Error),
}

#[derive(Debug, Error)]
pub enum PluginManagerError {
    #[error("plugin not found")]
    PluginNotFound,
    #[error("already enabled")]
    AlreadyEnabled,
    #[error("already disabled")]
    AlreadyDisabled,
    #[error("depends on '{dependency}' which is not enabled")]
    MissingDependency { dependency: String },
    #[error("still depended on by: {dependents:?}")]
    StillDependent { dependents: Box<[String]> },
    #[error("still enabled")]
    StillEnabled,
    #[error("wasmtime: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}
