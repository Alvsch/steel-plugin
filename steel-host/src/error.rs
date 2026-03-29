use std::{borrow::Cow, io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginContractError {
    #[error("plugin trapped: {0}")]
    Trap(wasmtime::Trap),

    #[error("{reason}")] // reason contains export name
    InvalidExport {
        name: &'static str,
        reason: Cow<'static, str>,
    },

    #[error("plugin provided a null pointer from allocator")]
    NullAllocation,

    #[error("plugin provided a null pointer during on_load")]
    NullLoadData,

    #[error("plugin provided an null pointer")]
    NullPointer,

    #[error("plugin provided an invalid plugin/method id")]
    InvalidId,

    #[error("plugin provided an out of bounds pointer")]
    OutOfBoundsPointer,

    #[error("wasm: {0}")]
    WasmError(wasmtime::Error),

    #[error("{0}")]
    Other(String),
}

impl From<wasmtime::Error> for PluginContractError {
    fn from(value: wasmtime::Error) -> Self {
        if let Some(trap) = value.downcast_ref::<wasmtime::Trap>() {
            return Self::Trap(*trap);
        }
        Self::WasmError(value)
    }
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin not found: {file_path}")]
    NotFound { file_path: PathBuf },

    #[error("invalid module: {0}")]
    InvalidModule(wasmtime::Error),

    #[error("contract error: {0}")]
    ContractError(#[from] PluginContractError),

    #[error("module instantiation error: {0}")]
    ModuleInstantiationError(wasmtime::Error),

    #[error("io error {0}")]
    Io(io::Error),
}
