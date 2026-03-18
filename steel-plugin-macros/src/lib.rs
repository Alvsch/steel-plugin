use crate::rules::validate;
use crate::{args::PluginMetaArgs, rules::FnRules};
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{Error, ItemFn, parse_macro_input};

mod args;
mod rules;

#[proc_macro]
pub fn plugin_meta(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as PluginMetaArgs);
    if args.name == "steel" {
        return Error::new(
            Span::call_site().into(),
            "The plugin name 'steel' is reserved",
        )
        .to_compile_error()
        .into();
    }

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

#[proc_macro_attribute]
pub fn on_enable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate(
        &FnRules {
            name: Some("on_enable"),
            require_pub: true,
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let stmts = &item.block.stmts;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_enable() {
            #(#stmts)*
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn on_disable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate(
        &FnRules {
            name: Some("on_disable"),
            require_pub: true,
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let stmts = &item.block.stmts;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_disable() {
            #(#stmts)*
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn rpc_export(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate(
        &FnRules {
            name: None,
            params: Some(&["& [u8]"]),
            ret: Some("Option < Vec < u8 > >"),
            require_pub: true,
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let inputs = &item.sig.inputs;
    let arg = inputs.first().unwrap();

    let fn_name = item.sig.ident;

    let stmts = &item.block.stmts;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #fn_name(data_ptr: u64) -> u64 {
            fn __impl(#arg) -> Option<Vec<u8>> {
                #(#stmts)*
            }

            let data_ptr = ::steel_plugin_sdk::utils::fat::FatPtr::unpack(data_ptr).unwrap();
            let data = unsafe {
                std::slice::from_raw_parts(data_ptr.ptr() as *mut u8, data_ptr.len() as usize)
            };

            let Some(return_data) = __impl(data) else {
                return 0;
            };
            let fat = ::steel_plugin_sdk::utils::fat::FatPtr::new(return_data.as_ptr() as u32, return_data.len() as u32).unwrap();
            std::mem::forget(return_data);
            fat.pack()
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn event_handler(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);

    let inputs = &item.sig.inputs;
    let arg = inputs.first().expect("function needs one parameter");
    let syn::FnArg::Typed(pat_type) = arg else {
        panic!("self parameters not supported");
    };
    let arg_type = &pat_type.ty;

    if let Err(err) = validate(
        &FnRules {
            require_pub: false,
            ret: Some(&format!("Option < {} >", quote! { #arg_type })),
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let name = &item.sig.ident;
    let stmts = &item.block.stmts;

    quote! {
        fn #name(packed: u64) -> u64 {
            #[inline(always)]
            fn __impl(#arg) -> Option<#arg_type> {
                #(#stmts)*
            }

            let fat = ::steel_plugin_sdk::utils::fat::FatPtr::unpack(packed).unwrap();
            let data = unsafe {
                std::slice::from_raw_parts(fat.ptr() as *mut u8, fat.len() as usize)
            };

            let event = ::rmp_serde::from_slice(data).unwrap();
            let Some(result) = __impl(event) else {
                return 0;
            };
            let result = ::rmp_serde::to_vec(&result).unwrap();
            let fat = ::steel_plugin_sdk::utils::fat::FatPtr::new(result.as_ptr() as u32, result.len() as u32).unwrap();
            std::mem::forget(result);
            fat.pack()
        }
    }
    .into()
}
