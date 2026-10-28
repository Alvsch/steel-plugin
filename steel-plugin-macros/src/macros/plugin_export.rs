use std::env;

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use semver::Version;
use steel_plugin_core::PluginMeta;
use syn::{Error, Ident};

use crate::PluginExportInput;

fn import_plugin_export() -> TokenStream {
    match crate_name("steel-plugin-sdk").expect("steel-plugin-sdk not found in Cargo.toml") {
        FoundCrate::Itself => quote!(crate::__export::c),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( ::#ident::__export )
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
    };

    if meta.name == "steel" {
        return Error::new(Span::call_site(), "The plugin name 'steel' is reserved")
            .to_compile_error();
    }

    let bytes: Vec<u8> = meta.serialize();
    let len = bytes.len();

    let export = import_plugin_export();
    quote! {
        const _: () = {
            #[unsafe(link_section = "steel-api::plugin::metadata")]
            #[used]
            static __PLUGIN_METADATA: [u8; #len] = [#(#bytes),*];

            #export::plugin_export!(#plugin);
        };
    }
}
