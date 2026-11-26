use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemFn, ReturnType};

use crate::sdk_path::sdk_export_path;

fn validate(item: &ItemFn) -> Result<(), Error> {
    if let Some(asyncness) = item.sig.asyncness.as_ref() {
        return Err(Error::new(asyncness.span(), "function must not be async"));
    }

    if item.sig.inputs.len() != 1 {
        return Err(Error::new(
            item.sig.inputs.span(),
            "function must have exactly 1 parameter(s)",
        ));
    }

    let arg = item
        .sig
        .inputs
        .first()
        .expect("one parameter checked above");
    let FnArg::Typed(pat_type) = arg else {
        return Err(Error::new(
            arg.span(),
            "function must not have a self parameter",
        ));
    };
    let ty = &pat_type.ty;
    let arg_type = quote!(#ty).to_string();
    if arg_type != "& [u8]" {
        return Err(Error::new(
            pat_type.ty.span(),
            format!("expected parameter type '& [u8]', found '{arg_type}'"),
        ));
    }

    match &item.sig.output {
        ReturnType::Default => Err(Error::new(
            item.sig.span(),
            "function must return 'Option < Vec < u8 > >'",
        )),
        ReturnType::Type(_, ty) if quote!(#ty).to_string() != "Option < Vec < u8 > >" => {
            Err(Error::new(
                ty.span(),
                format!(
                    "expected return type 'Option < Vec < u8 > >', found '{}'",
                    quote!(#ty)
                ),
            ))
        }
        ReturnType::Type(_, _) => Ok(()),
    }
}

pub(crate) fn expand(item: ItemFn) -> TokenStream {
    if let Err(error) = validate(&item) {
        return error.to_compile_error();
    }

    let fn_name = item.sig.ident;
    let arg = match item
        .sig
        .inputs
        .first()
        .expect("function needs one argument")
    {
        FnArg::Receiver(_) => panic!("function argument cant be self"),
        FnArg::Typed(pat_type) => &pat_type.pat,
    };
    let stmts = &item.block.stmts;

    let export = sdk_export_path();
    quote! {
        #export::submit! {
            #export::RpcMethod {
                name: stringify!(#fn_name),
                function: |#arg| {
                    #(#stmts)*
                },
            }
        }

        #[cfg(test)]
        mod tests {
            use syn::{ItemFn, parse_quote};

            use super::validate;

            #[test]
            fn accepts_rpc_signature() {
                let item: ItemFn = parse_quote! {
                    fn rpc_handler(data: &[u8]) -> Option<Vec<u8>> {
                        let _ = data;
                        None
                    }
                };

                assert!(validate(&item).is_ok());
            }

            #[test]
            fn rejects_wrong_rpc_parameter_type() {
                let item: ItemFn = parse_quote! {
                    fn rpc_handler(data: &str) -> Option<Vec<u8>> {
                        let _ = data;
                        None
                    }
                };

                let error = validate(&item).expect_err("wrong parameter type should fail");
                assert!(error.to_string().contains("expected parameter type"));
            }
        }
    }
}
