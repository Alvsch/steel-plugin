use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::utils::rules::{FnRules, validate};

pub(crate) fn event_handler(item: ItemFn) -> TokenStream {
    let arg = &item
        .sig
        .inputs
        .first()
        .expect("function needs one parameter");
    let syn::FnArg::Typed(pat_type) = arg else {
        panic!("self parameters not supported");
    };
    let arg_type = &pat_type.ty;
    let stmts = &item.block.stmts;

    if let Err(err) = validate(
        &FnRules {
            require_pub: false,
            ret: Some(&format!("Option < {} >", quote! { #arg_type })),
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error();
    }

    let topic: TokenStream = format!("b\"{}\"", quote! { #arg_type }).parse().unwrap();

    quote! {
        ::inventory::submit! {
            ::steel_plugin_sdk::Exported {
                kind: ::steel_plugin_sdk::ExportedKind::Event(::steel_plugin_sdk::event::hash_topic(#topic)),
                func: |packed| {
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
                },
            }
        }
    }
}
