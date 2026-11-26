use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, FnArg, GenericArgument, ItemFn, PathArguments, ReturnType, Type, spanned::Spanned,
};

use crate::sdk_path::sdk_export_path;

enum RpcHandler {
    Raw {
        returns_bytes: bool,
    },
    Typed {
        request: Box<Type>,
        response: Option<Box<Type>>,
    },
}

fn is_unit(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
}

fn is_byte_slice_reference(ty: &Type) -> bool {
    let Type::Reference(reference) = ty else {
        return false;
    };
    if reference.mutability.is_some() {
        return false;
    }

    let Type::Slice(slice) = &*reference.elem else {
        return false;
    };
    is_u8(&slice.elem)
}

fn path_ends_in(ty: &Type, name: &str) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == name))
}

fn type_argument<'a>(ty: &'a Type, name: &str) -> Option<&'a Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let mut arguments = arguments.args.iter();
    let Some(GenericArgument::Type(argument)) = arguments.next() else {
        return None;
    };
    if arguments.next().is_some() {
        return None;
    }
    Some(argument)
}

fn is_u8(ty: &Type) -> bool {
    path_ends_in(ty, "u8")
}

fn is_optional_bytes(ty: &Type) -> bool {
    type_argument(ty, "Option")
        .and_then(|ty| type_argument(ty, "Vec"))
        .is_some_and(is_u8)
}

fn validate(item: &ItemFn) -> Result<RpcHandler, Error> {
    if let Some(asyncness) = item.sig.asyncness.as_ref() {
        return Err(Error::new(asyncness.span(), "function must not be async"));
    }
    if !item.sig.generics.params.is_empty() {
        return Err(Error::new(
            item.sig.generics.span(),
            "RPC functions must not be generic",
        ));
    }
    if item.sig.inputs.len() != 1 {
        return Err(Error::new(
            item.sig.inputs.span(),
            "function must have exactly one parameter",
        ));
    }

    let arg = item
        .sig
        .inputs
        .first()
        .expect("one parameter checked above");
    let FnArg::Typed(argument) = arg else {
        return Err(Error::new(
            arg.span(),
            "function must not have a self parameter",
        ));
    };

    if is_byte_slice_reference(&argument.ty) {
        let returns_bytes = match &item.sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, ty) if is_unit(ty) => false,
            ReturnType::Type(_, ty) if is_optional_bytes(ty) => true,
            ReturnType::Type(_, ty) => {
                return Err(Error::new(
                    ty.span(),
                    "raw RPC functions must return `()` or `Option<Vec<u8>>`",
                ));
            }
        };
        return Ok(RpcHandler::Raw { returns_bytes });
    }

    if matches!(&*argument.ty, Type::Reference(_)) {
        return Err(Error::new(
            argument.ty.span(),
            "typed RPC request parameters must be owned values",
        ));
    }

    let response = match &item.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) if is_unit(ty) => None,
        ReturnType::Type(_, ty) => Some((**ty).clone()),
    };

    Ok(RpcHandler::Typed {
        request: Box::new((*argument.ty).clone()),
        response: response.map(Box::new),
    })
}

pub(crate) fn expand(item: ItemFn) -> TokenStream {
    let handler = match validate(&item) {
        Ok(handler) => handler,
        Err(error) => return error.to_compile_error(),
    };

    let fn_name = &item.sig.ident;
    let FnArg::Typed(argument) = item.sig.inputs.first().expect("one parameter checked") else {
        unreachable!("validation rejects receiver parameters");
    };
    let arg_name = &argument.pat;
    let arg_type = &argument.ty;
    let output = &item.sig.output;
    let stmts = &item.block.stmts;
    let implementation = syn::Ident::new("__steel_rpc_export", Span::mixed_site());
    let wire_data = syn::Ident::new("__steel_rpc_wire_data", Span::mixed_site());
    let export = sdk_export_path();

    match handler {
        RpcHandler::Raw { returns_bytes } => {
            let result = if returns_bytes {
                quote! { Ok(#implementation(#wire_data)) }
            } else {
                quote! {
                    #implementation(#wire_data);
                    Ok(None)
                }
            };
            quote! {
                #export::submit! {
                    #export::RpcMethod {
                        name: stringify!(#fn_name),
                        function: |#wire_data| {
                            #[allow(clippy::unnecessary_wraps)]
                            fn #implementation(#arg_name: &[u8]) #output {
                                #(#stmts)*
                            }
                            #result
                        },
                    }
                }
            }
        }
        RpcHandler::Typed { request, response } => {
            let request = &*request;
            let encode = if response.is_some() {
                quote! {
                    let response = #implementation(request);
                    #export::encode_rpc_response(&response)
                }
            } else {
                quote! {
                    #implementation(request);
                    Ok(None)
                }
            };
            quote! {
                #export::submit! {
                    #export::RpcMethod {
                        name: stringify!(#fn_name),
                        function: |#wire_data| {
                            let request = #export::decode_rpc_request::<#request>(#wire_data)?;
                            fn #implementation(#arg_name: #arg_type) #output {
                                #(#stmts)*
                            }
                            #encode
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RpcHandler, validate};
    use syn::{ItemFn, parse_quote};

    #[test]
    fn accepts_raw_no_response_function() {
        let item: ItemFn = parse_quote! {
            fn notify(data: &[u8]) {
                let _ = data;
            }
        };
        assert!(matches!(
            validate(&item),
            Ok(RpcHandler::Raw {
                returns_bytes: false
            })
        ));
    }

    #[test]
    fn accepts_raw_response_function() {
        let item: ItemFn = parse_quote! {
            fn echo(data: &[u8]) -> std::option::Option<std::vec::Vec<u8>> {
                Some(data.to_vec())
            }
        };
        assert!(matches!(
            validate(&item),
            Ok(RpcHandler::Raw {
                returns_bytes: true
            })
        ));
    }

    #[test]
    fn accepts_typed_unit_and_response_functions() {
        let no_response: ItemFn = parse_quote! {
            fn notify(request: Request) {
                let _ = request;
            }
        };
        let response: ItemFn = parse_quote! {
            fn greet(request: Request) -> Response {
                let _ = request;
                Response
            }
        };

        assert!(matches!(
            validate(&no_response),
            Ok(RpcHandler::Typed { response: None, .. })
        ));
        assert!(matches!(
            validate(&response),
            Ok(RpcHandler::Typed {
                response: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn rejects_borrowed_typed_requests() {
        let item: ItemFn = parse_quote! {
            fn greet(request: &Request) -> Response {
                let _ = request;
                Response
            }
        };
        let Err(error) = validate(&item) else {
            panic!("borrowed request should be rejected");
        };
        assert!(error.to_string().contains("owned values"));
    }

    #[test]
    fn rejects_unsupported_raw_response() {
        let item: ItemFn = parse_quote! {
            fn echo(data: &[u8]) -> bool {
                !data.is_empty()
            }
        };
        let Err(error) = validate(&item) else {
            panic!("bool raw response should be rejected");
        };
        assert!(error.to_string().contains("Option<Vec<u8>>"));
    }
}
