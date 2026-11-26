use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemFn, ReturnType};

use crate::sdk_path::sdk_export_path;

#[derive(Debug, PartialEq, Eq)]
enum RpcReturn {
    Unit,
    OptionalBytes,
}

fn validate(item: &ItemFn) -> Result<RpcReturn, Error> {
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
        ReturnType::Default => Ok(RpcReturn::Unit),
        ReturnType::Type(_, ty) => match quote!(#ty).to_string().as_str() {
            "()" => Ok(RpcReturn::Unit),
            "Option < Vec < u8 > >" => Ok(RpcReturn::OptionalBytes),
            found => Err(Error::new(
                ty.span(),
                format!("expected return type '()' or 'Option<Vec<u8>>', found '{found}'"),
            )),
        },
    }
}

pub(crate) fn expand(item: ItemFn) -> TokenStream {
    let output = match validate(&item) {
        Ok(output) => output,
        Err(error) => return error.to_compile_error(),
    };

    let fn_name = &item.sig.ident;
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
    match output {
        RpcReturn::Unit => {
            let data = syn::Ident::new("__steel_rpc_data", proc_macro2::Span::mixed_site());
            quote! {
                #export::submit! {
                    #export::RpcMethod {
                        name: stringify!(#fn_name),
                        function: |#data| {
                            fn __steel_rpc_export(#arg: &[u8]) {
                                #(#stmts)*
                            }
                            __steel_rpc_export(#data);
                            None
                        }
                    }
                }
            }
        }
        RpcReturn::OptionalBytes => quote! {
            #export::submit! {
                #export::RpcMethod {
                    name: stringify!(#fn_name),
                    function: |#arg| {
                        #(#stmts)*
                    },
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{RpcReturn, validate};
    use syn::{ItemFn, parse_quote};

    #[test]
    fn accepts_implicit_unit_return() {
        let item: ItemFn = parse_quote! {
            fn notify(data: &[u8]) {
                let _ = data;
            }
        };

        assert!(matches!(validate(&item), Ok(RpcReturn::Unit)));
    }

    #[test]
    fn accepts_explicit_unit_return() {
        let item: ItemFn = parse_quote! {
            fn notify(data: &[u8]) -> () {
                let _ = data;
            }
        };

        assert!(matches!(validate(&item), Ok(RpcReturn::Unit)));
    }

    #[test]
    fn accepts_optional_byte_response() {
        let item: ItemFn = parse_quote! {
            fn echo(data: &[u8]) -> Option<Vec<u8>> {
                Some(data.to_vec())
            }
        };

        assert!(matches!(validate(&item), Ok(RpcReturn::OptionalBytes)));
    }

    #[test]
    fn rejects_other_return_types() {
        let item: ItemFn = parse_quote! {
            fn invalid(data: &[u8]) -> bool {
                !data.is_empty()
            }
        };

        let error = validate(&item).expect_err("bool response should be rejected");
        assert!(error.to_string().contains("Option<Vec<u8>>"));
    }
}
