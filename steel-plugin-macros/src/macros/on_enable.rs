use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::utils::rules::{FnRules, validate};

pub(crate) fn on_enable(item: ItemFn) -> TokenStream {
    if let Err(err) = validate(
        &FnRules {
            name: Some("on_enable"),
            require_pub: true,
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error();
    }

    let stmts = &item.block.stmts;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_enable() {
            #(#stmts)*
        }
    }
}
