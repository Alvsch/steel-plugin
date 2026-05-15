use std::{env, path::PathBuf};

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use semver::Version;
use steel_plugin_core::PluginMeta;
use syn::{Error, Ident};

use crate::PluginExportInput;

fn import_plugin_api() -> TokenStream {
    match crate_name("steel-plugin-sdk").expect("steel-plugin-sdk not found in Cargo.toml") {
        FoundCrate::Itself => quote!(crate::component::exports::host::plugin_sdk::plugin_api),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident::component::exports::host::plugin_sdk::plugin_api )
        }
    }
}

pub fn plugin_export(PluginExportInput { plugin, meta }: PluginExportInput) -> TokenStream {
    let meta = meta.unwrap_or_default();

    let meta = PluginMeta {
        name: env::var("CARGO_PKG_NAME").expect("no name"),
        description: env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default(),
        version: Version::parse(&env::var("CARGO_PKG_VERSION").unwrap_or_default())
            .expect("invalid version"),
        authors: env::var("CARGO_PKG_AUTHORS")
            .map(|authors| authors.split(':').map(ToString::to_string).collect())
            .unwrap_or_default(),
        depends: meta.depends,
        api_version: steel_plugin_core::STEEL_API_VERSION,
        file_path: PathBuf::new(),
    };

    if meta.name == "steel" {
        return Error::new(Span::call_site(), "The plugin name 'steel' is reserved")
            .to_compile_error();
    }

    let bytes: Vec<u8> = meta.serialize();
    let len = bytes.len();

    let import = import_plugin_api();
    quote! {
        const _: () = {
            #[unsafe(link_section = "steel-api::plugin::metadata")]
            #[used]
            static __PLUGIN_METADATA: [u8; #len] = [#(#bytes),*];

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#on-enable")]
            unsafe extern "C" fn export_on_enable() {
                unsafe {
                    #import::_export_on_enable_cabi::<#plugin>()
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#on-disable")]
            unsafe extern "C" fn export_on_disable() {
                unsafe {
                    #import::_export_on_disable_cabi::<#plugin>()
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#on-load")]
            unsafe extern "C" fn export_on_load() -> *mut u8 {
                unsafe {
                    #import::_export_on_load_cabi::<#plugin>()
                }
            }

            #[unsafe(export_name = "cabi_post_host:plugin-sdk/plugin-api@0.1.0#on-load")]
            unsafe extern "C" fn _post_return_on_load(arg0: *mut u8) {
                unsafe {
                    #import::__post_return_on_load::<#plugin>(arg0)
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#rpc")]
            unsafe extern "C" fn export_rpc(arg0: i32, arg1: *mut u8, arg2: usize) -> *mut u8 {
                unsafe {
                    #import::_export_rpc_cabi::<#plugin>(arg0, arg1, arg2)
                }
            }

            #[unsafe(export_name = "cabi_post_host:plugin-sdk/plugin-api@0.1.0#rpc")]
            unsafe extern "C" fn _post_return_rpc(arg0: *mut u8) {
                unsafe {
                    #import::__post_return_rpc::<#plugin>(arg0)
                }
            }

            #[unsafe(export_name = "host:plugin-sdk/plugin-api@0.1.0#event-handler")]
            unsafe extern "C" fn export_event_handler(arg0: i32, arg1: *mut u8, arg2: usize) {
                unsafe {
                    #import::_export_event_handler_cabi::<#plugin>(arg0, arg1, arg2)
                }
            }
        };
    }
}
