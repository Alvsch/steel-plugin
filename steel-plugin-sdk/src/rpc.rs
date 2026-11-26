//! Host RPC resolution and plugin-side RPC export support.
//!
//! A plugin can resolve methods only from plugins named in its `depends` metadata.
//! RPC exports may return `()` for no response or `Option<Vec<u8>>` for an
//! optional byte response. [`call`] resolves and dispatches in one step.
use crate::sdk::rpc;
use thiserror::Error;

/// Why an RPC could not be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CallError {
    /// The target is unavailable or the caller has not declared it as a dependency.
    #[error("RPC target is unavailable")]
    MethodUnavailable,
    /// The target plugin does not export the requested method.
    #[error("RPC method is unknown")]
    UnknownMethod,
}

pub use crate::sdk::rpc::resolve_method;

/// Resolve and dispatch an RPC method on a declared dependency.
///
/// `Ok(None)` means the call succeeded without a response. Resolution
/// failures and unknown methods are returned as distinct errors.
pub fn call(
    plugin_name: &str,
    method_name: &str,
    data: &[u8],
) -> Result<Option<Vec<u8>>, CallError> {
    let method = resolve_method(plugin_name, method_name).ok_or(CallError::MethodUnavailable)?;
    method.dispatch(data).map_err(|error| match error {
        rpc::RpcError::UnknownMethod => CallError::UnknownMethod,
    })
}
