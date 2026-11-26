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

/// Register an RPC function taking `&[u8]` and returning either `()` or
/// `Option<Vec<u8>>`. Unit-returning functions are exposed as no-response RPCs.
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
