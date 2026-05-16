use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemFn};

use crate::utils::rules::{FnRules, validate};

fn import_rpc_export() -> TokenStream {
    match crate_name("steel-plugin-sdk").expect("steel-plugin-sdk not found in Cargo.toml") {
        FoundCrate::Itself => quote!(crate::rpc::export),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident::rpc::export )
        }
    }
}

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
    let arg = match item
        .sig
        .inputs
        .first()
        .expect("function needs one argument &[u8]")
    {
        syn::FnArg::Receiver(_) => panic!("function argument cant be self"),
        syn::FnArg::Typed(pat_type) => &pat_type.pat,
    };
    let stmts = &item.block.stmts;

    let rpc = import_rpc_export();
    quote! {
        #rpc::submit! {
            #rpc::RpcMethod {
                name: stringify!(#fn_name),
                function: |#arg| {
                    #(#stmts)*
                },
            }
        }
    }
}
