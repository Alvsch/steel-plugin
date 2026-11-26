//! Procedural macros used by steel-plugin-sdk.

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

mod event_handler;
mod plugin_export;
mod rpc_export;
mod sdk_path;

#[proc_macro]
pub fn plugin_export(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as plugin_export::PluginExportInput);
    plugin_export::expand(input).into()
}

/// Register an RPC function exported by the plugin.
///
/// Typed handlers take one owned request type that implements
/// `serde::Deserialize`. A serializable return value becomes the response;
/// returning `()` means the method has no response. Payloads use named
/// `MessagePack` fields. The plugin crate must depend on `serde` with its
/// `derive` feature enabled.
///
/// ```rust,ignore
/// use steel_plugin_sdk::{
///     rpc_export,
///     serde::{Deserialize, Serialize},
/// };
///
/// #[derive(Deserialize)]
/// struct GreetRequest {
///     name: String,
/// }
///
/// #[derive(Serialize)]
/// struct GreetResponse {
///     message: String,
/// }
///
/// #[rpc_export]
/// fn greet(request: GreetRequest) -> GreetResponse {
///     GreetResponse {
///         message: format!("hello {}", request.name),
///     }
/// }
///
/// #[derive(Deserialize)]
/// struct RefreshRequest {
///     key: String,
/// }
///
/// #[rpc_export]
/// fn refresh(request: RefreshRequest) {
///     let _key = request.key;
///     // Update the provider's cache.
/// }
/// ```
///
/// Raw handlers can receive bytes directly. Return `()` for a no-response
/// method or `Option<Vec<u8>>` for an optional byte response:
///
/// ```rust,ignore
/// use steel_plugin_sdk::rpc_export;
///
/// #[rpc_export]
/// fn echo(data: &[u8]) -> Option<Vec<u8>> {
///     Some(data.to_vec())
/// }
///
/// #[rpc_export]
/// fn log_request(data: &[u8]) {
///     eprintln!("received {} bytes", data.len());
/// }
/// ```
///
/// Handlers must be synchronous, non-generic functions with exactly one
/// parameter. Typed requests must be owned; borrowed request types are not
/// supported.
#[proc_macro_attribute]
pub fn rpc_export(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    rpc_export::expand(item).into()
}

#[proc_macro_attribute]
pub fn event_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    let priority = if args.is_empty() {
        0
    } else {
        parse_macro_input!(args as event_handler::EventPriority).0
    };
    event_handler::expand(item, priority).into()
}
