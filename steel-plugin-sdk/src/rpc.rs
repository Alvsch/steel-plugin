//! Raw and MessagePack-typed RPC calls.
//!
//! A plugin can resolve methods only from plugins named in its `depends` metadata.
//! Use [`call`] or [`Method::dispatch`] for raw bytes, [`request`] for typed
//! request/response calls, and [`notify`] for typed no-response calls.

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::sdk::rpc::{self, Method};

/// Why an RPC could not be completed.
#[derive(Debug, Error)]
pub enum CallError {
    /// The target is unavailable or the caller has not declared it as a dependency.
    #[error("RPC target is unavailable")]
    MethodUnavailable,
    /// The target plugin does not export the requested method.
    #[error("RPC method is unknown")]
    UnknownMethod,
    /// The provider could not decode the request payload.
    #[error("RPC provider rejected the request payload")]
    InvalidRequest,
    /// The provider could not serialize its response.
    #[error("RPC provider could not serialize its response")]
    ResponseSerializationFailed,
    /// The local plugin could not serialize the request.
    #[error("failed to serialize RPC request: {0}")]
    RequestSerialization(#[source] rmp_serde::encode::Error),
    /// The local plugin could not deserialize the response.
    #[error("failed to deserialize RPC response: {0}")]
    ResponseDeserialization(#[source] rmp_serde::decode::Error),
    /// The RPC succeeded but returned no response when one was expected.
    #[error("RPC completed without a response")]
    NoResponse,
    /// The RPC returned a response when none was expected.
    #[error("RPC returned an unexpected response")]
    UnexpectedResponse,
}

pub use crate::sdk::rpc::resolve_method;

/// Resolve and dispatch an RPC method using raw bytes.
///
/// `Ok(None)` means the call succeeded without a response.
pub fn call(
    plugin_name: &str,
    method_name: &str,
    data: &[u8],
) -> Result<Option<Vec<u8>>, CallError> {
    let method = resolve_method(plugin_name, method_name).ok_or(CallError::MethodUnavailable)?;
    method.dispatch(data).map_err(map_rpc_error)
}

/// Resolve a method, serialize a request with named `MessagePack` fields, and
/// deserialize its response.
pub fn request<Request: Serialize, Response: DeserializeOwned>(
    plugin_name: &str,
    method_name: &str,
    request: &Request,
) -> Result<Response, CallError> {
    let method = resolve_method(plugin_name, method_name).ok_or(CallError::MethodUnavailable)?;
    method.request(request)
}

/// Resolve a method and send a serialized request that is expected to have no
/// response.
pub fn notify<Request: Serialize>(
    plugin_name: &str,
    method_name: &str,
    request: &Request,
) -> Result<(), CallError> {
    let method = resolve_method(plugin_name, method_name).ok_or(CallError::MethodUnavailable)?;
    method.notify(request)
}

impl Method {
    /// Dispatch raw bytes to this resolved method.
    pub fn dispatch_raw(&self, data: &[u8]) -> Result<Option<Vec<u8>>, CallError> {
        self.dispatch(data).map_err(map_rpc_error)
    }

    /// Serialize a request with named `MessagePack` fields and deserialize the
    /// required response.
    pub fn request<Request: Serialize, Response: DeserializeOwned>(
        &self,
        request: &Request,
    ) -> Result<Response, CallError> {
        let data = encode_request(request)?;
        let response = self
            .dispatch(&data)
            .map_err(map_rpc_error)?
            .ok_or(CallError::NoResponse)?;
        rmp_serde::from_slice(&response).map_err(CallError::ResponseDeserialization)
    }

    /// Serialize a request with named `MessagePack` fields and require no
    /// response from the method.
    pub fn notify<Request: Serialize>(&self, request: &Request) -> Result<(), CallError> {
        let data = encode_request(request)?;
        let response = self.dispatch(&data).map_err(map_rpc_error)?;
        if response.is_some() {
            return Err(CallError::UnexpectedResponse);
        }
        Ok(())
    }
}

fn encode_request<Request: Serialize>(request: &Request) -> Result<Vec<u8>, CallError> {
    rmp_serde::to_vec_named(request).map_err(CallError::RequestSerialization)
}

const fn map_rpc_error(error: rpc::RpcError) -> CallError {
    match error {
        rpc::RpcError::UnknownMethod => CallError::UnknownMethod,
        rpc::RpcError::InvalidRequest => CallError::InvalidRequest,
        rpc::RpcError::ResponseSerializationFailed => CallError::ResponseSerializationFailed,
    }
}

#[doc(hidden)]
pub fn decode_rpc_request<Request: DeserializeOwned>(
    data: &[u8],
) -> Result<Request, rpc::RpcError> {
    rmp_serde::from_slice(data).map_err(|_| rpc::RpcError::InvalidRequest)
}

#[doc(hidden)]
pub fn encode_rpc_response<Response: Serialize>(
    response: &Response,
) -> Result<Option<Vec<u8>>, rpc::RpcError> {
    rmp_serde::to_vec_named(response)
        .map(Some)
        .map_err(|_| rpc::RpcError::ResponseSerializationFailed)
}

#[cfg(test)]
mod tests {
    use serde::{Serialize, Serializer};

    use super::{CallError, decode_rpc_request, encode_request, encode_rpc_response};
    use crate::sdk::rpc;

    struct Unserializable;

    impl Serialize for Unserializable {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom(
                "intentional serialization failure",
            ))
        }
    }

    #[test]
    fn malformed_typed_request_becomes_rpc_error() {
        assert!(matches!(
            decode_rpc_request::<String>(&[0xc1]),
            Err(rpc::RpcError::InvalidRequest)
        ));
    }

    #[test]
    fn local_request_serialization_error_is_preserved() {
        assert!(matches!(
            encode_request(&Unserializable),
            Err(CallError::RequestSerialization(_))
        ));
    }

    #[test]
    fn provider_response_serialization_error_becomes_rpc_error() {
        assert!(matches!(
            encode_rpc_response(&Unserializable),
            Err(rpc::RpcError::ResponseSerializationFailed)
        ));
    }
}
