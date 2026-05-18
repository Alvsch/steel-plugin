use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemFn, Type};

use crate::utils::rules::{FnRules, validate};

fn import_export() -> TokenStream {
    match crate_name("steel-plugin-sdk").expect("steel-plugin-sdk not found in Cargo.toml") {
        FoundCrate::Itself => quote!(crate::__export),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident::__export )
        }
    }
}

pub(crate) fn event_handler(item: ItemFn, priority: i8) -> TokenStream {
    let arg = &item
        .sig
        .inputs
        .first()
        .expect("function needs one parameter");
    let syn::FnArg::Typed(pat_type) = arg else {
        panic!("self parameters not supported");
    };
    let arg_type = &pat_type.ty;
    let Type::Reference(type_ref) = &**arg_type else {
        panic!("no ref");
    };
    let elem = &type_ref.elem;

    let stmts = &item.block.stmts;

    if let Err(err) = validate(
        &FnRules {
            require_pub: false,
            ret: None,
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error();
    }

    let event_module = import_export();
    quote! {
        ::steel_plugin_sdk::__export::submit! {
            ::steel_plugin_sdk::event::EventHandler {
                id: 0,
                priority: #priority,
                function: |event: &mut #event_module::Event| {
                    #[inline(always)]
                    fn __impl(event: #arg_type) {
                        #(#stmts)*
                    }

                    if let #event_module::Event::#elem(event) = event {
                        __impl(event);
                    }
                },
            }
        }
    }
}
