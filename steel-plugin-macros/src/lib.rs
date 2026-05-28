use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::utils::{event_priority::EventPriority, export_input::PluginExportInput};

mod macros;
pub(crate) mod utils;

#[proc_macro]
pub fn plugin_export(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as PluginExportInput);
    macros::plugin_export(input).into()
}

#[proc_macro_attribute]
pub fn rpc_export(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    macros::rpc_export(item).into()
}

#[proc_macro_attribute]
pub fn event_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    let priority = if args.is_empty() {
        0
    } else {
        parse_macro_input!(args as EventPriority).0
    };
    macros::event_handler(item, priority).into()
}
