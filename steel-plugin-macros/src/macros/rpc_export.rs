use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::utils::rules::{FnRules, validate};

pub(crate) fn rpc_export(item: ItemFn) -> TokenStream {
    if let Err(err) = validate(
        &FnRules {
            name: None,
            params: Some(&["& [u8]"]),
            ret: Some("Option < Vec < u8 > >"),
            require_pub: false,
        },
        &item,
    ) {
        return err.to_compile_error();
    }

    let fn_name = item.sig.ident;
    let inputs = &item.sig.inputs;
    let arg = inputs.first().unwrap();
    let stmts = &item.block.stmts;

    quote! {
        ::steel_plugin_sdk::export::submit! {
            ::steel_plugin_sdk::export::Exported {
                kind: ::steel_plugin_sdk::export::ExportedKind::Rpc {
                    export_name: std::borrow::Cow::Borrowed(stringify!(#fn_name)),
                },
                func: |data_ptr| {
                    #[inline(always)]
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
                },
            }
        }
    }
}
