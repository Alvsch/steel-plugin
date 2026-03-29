use std::{env, path::PathBuf};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use semver::Version;
use steel_plugin_core::PluginMeta;
use syn::Error;

use crate::PluginMetaArgs;

pub fn plugin_meta(input: PluginMetaArgs) -> TokenStream {
    let meta = PluginMeta {
        name: env::var("CARGO_PKG_NAME").expect("no name"),
        description: env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default(),
        version: Version::parse(&env::var("CARGO_PKG_VERSION").unwrap_or_default())
            .expect("invalid version"),
        depends: input.depends,
        api_version: steel_plugin_core::STEEL_API_VERSION,
        file_path: PathBuf::new(),
    };

    if meta.name == "steel" {
        return Error::new(Span::call_site(), "The plugin name 'steel' is reserved")
            .to_compile_error();
    }

    let bytes: Vec<u8> = meta.serialize();
    let len = bytes.len();

    quote! {
        #[unsafe(link_section = "plugin_meta")]
        #[used]
        pub static __PLUGIN_META_SECTION: [u8; #len] = [#(#bytes),*];

        #[unsafe(no_mangle)]
        pub extern "C" fn on_load() -> u64 {
            let slice = ::steel_plugin_sdk::export::iter::<::steel_plugin_sdk::export::Exported>().cloned().map(::steel_plugin_sdk::export::ExportedId::from).collect::<Vec<_>>();
            let bytes = ::rmp_serde::to_vec(&slice).unwrap();
            let fat = ::steel_plugin_sdk::utils::fat::FatPtr::new(bytes.as_ptr() as u32, bytes.len() as u32).unwrap();
            std::mem::forget(bytes);
            fat.pack()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn alloc(len: u32) -> u32 {
            let layout = std::alloc::Layout::from_size_align(len as usize, 1).unwrap();
            unsafe { std::alloc::alloc(layout) as u32 }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn dealloc(ptr: u32, len: u32) {
            let layout = std::alloc::Layout::from_size_align(len as usize, 1).unwrap();
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }
    }
}
