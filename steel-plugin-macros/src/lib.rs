use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::utils::args::PluginMetaArgs;

mod macros;
pub(crate) mod utils;

#[proc_macro]
pub fn plugin_meta(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as PluginMetaArgs);
    macros::plugin_meta(input).into()
}

#[proc_macro_attribute]
pub fn on_enable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    macros::on_enable(item).into()
}

#[proc_macro_attribute]
pub fn on_disable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    macros::on_disable(item).into()
}

#[proc_macro_attribute]
pub fn rpc_export(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    macros::rpc_export(item).into()
}

#[proc_macro_attribute]
pub fn event_handler(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    macros::event_handler(item).into()
}
