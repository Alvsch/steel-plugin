use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::utils::rules::{FnRules, validate};

pub(crate) fn on_disable(item: ItemFn) -> TokenStream {
    if let Err(err) = validate(
        &FnRules {
            name: Some("on_disable"),
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
        pub extern "C" fn on_disable() {
            #(#stmts)*
        }
    }
}
