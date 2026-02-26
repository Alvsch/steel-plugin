use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, Visibility, parse_macro_input, spanned::Spanned};

use crate::args::PluginMetaArgs;

mod args;

#[proc_macro]
pub fn plugin_meta(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as PluginMetaArgs);

    let bytes: Vec<u8> = args.serialize();
    let len = bytes.len();

    quote! {
        #[unsafe(link_section = "plugin_meta")]
        #[used]
        pub static __PLUGIN_META_SECTION: [u8; #len] = [#(#bytes),*];

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
    .into()
}

fn validate(name: &str, item: &ItemFn) -> Result<(), Error> {
    let ident = &item.sig.ident;
    let inputs = &item.sig.inputs;
    let output = &item.sig.output;
    let asyncness = &item.sig.asyncness;
    let vis = &item.vis;

    if ident != name {
        return Err(Error::new(
            ident.span(),
            format!("function must be named '{name}'"),
        ));
    }

    if let Some(arg) = inputs.first() {
        return Err(Error::new(arg.span(), "function must have no parameters"));
    }

    match output {
        syn::ReturnType::Type(_, ty) => {
            return Err(Error::new(ty.span(), "function must have no return type"));
        }
        syn::ReturnType::Default => (),
    }

    if let Some(asyncness) = asyncness.as_ref() {
        return Err(Error::new(asyncness.span(), "function must not be async"));
    }

    if !matches!(vis, Visibility::Public(_)) {
        return Err(Error::new(vis.span(), "function must be public"));
    }

    Ok(())
}

#[proc_macro_attribute]
pub fn on_enable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate("on_enable", &item) {
        return err.to_compile_error().into();
    }

    let block = &item.block;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_enable() {
            #block
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn on_disable(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate("on_disable", &item) {
        return err.to_compile_error().into();
    }

    let block = &item.block;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_disable() {
            #block
        }
    }
    .into()
}
