use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use steel_plugin_core::fnv1a_32;
use syn::spanned::Spanned;
use syn::{
    Error, Ident, ItemFn, LitInt, ReturnType, Token, Type,
    parse::{Parse, ParseStream},
};

use crate::sdk_path::sdk_export_path;

fn validate(item: &ItemFn) -> Result<(), Error> {
    if let Some(asyncness) = item.sig.asyncness.as_ref() {
        return Err(Error::new(asyncness.span(), "function must not be async"));
    }

    if let ReturnType::Type(_, ty) = &item.sig.output {
        return Err(Error::new(ty.span(), "function must have no return type"));
    }

    Ok(())
}

#[derive(Debug)]
pub(crate) struct EventPriority(pub(crate) i8);

impl Parse for EventPriority {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident != "priority" {
            return Err(syn::Error::new(
                ident.span(),
                "unknown argument, expected `priority`",
            ));
        }
        let _: Token![=] = input.parse()?;
        let lit: LitInt = input.parse()?;
        let priority = lit
            .base10_parse::<i8>()
            .map_err(|_| syn::Error::new(lit.span(), "priority must be a valid i8"))?;
        if !input.is_empty() {
            return Err(input.error("unexpected token, `priority` is the only allowed argument"));
        }

        Ok(Self(priority))
    }
}

pub(crate) fn expand(item: ItemFn, priority: i8) -> TokenStream {
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

    if let Err(error) = validate(&item) {
        return error.to_compile_error();
    }

    let export = sdk_export_path();
    let topic_id = fnv1a_32(elem.to_token_stream().to_string().as_bytes());
    quote! {
        #export::submit! {
            #export::EventHandler {
                topic_id: #export::TopicId(#topic_id),
                priority: #priority,
                function: |event: &mut #export::Event| {
                    #[inline(always)]
                    fn __impl(event: #arg_type) {
                        #(#stmts)*
                    }

                    if let #export::Event::#elem(event) = event {
                        __impl(event);
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventPriority;
    use syn::{ItemFn, parse_str};

    #[test]
    fn parses_event_priority() {
        let priority: EventPriority = parse_str("priority = -1").expect("priority should parse");
        assert_eq!(priority.0, -1);
    }

    #[test]
    fn rejects_out_of_range_priority() {
        let error =
            parse_str::<EventPriority>("priority = 128").expect_err("out of range priority");
        assert!(error.to_string().contains("priority must be a valid i8"));
    }

    #[test]
    fn rejects_async_event_handler() {
        let item: ItemFn = syn::parse_quote! {
            async fn on_event(event: &mut Event) {
                let _ = event;
            }
        };

        let error = super::validate(&item).expect_err("async handler should be rejected");
        assert!(error.to_string().contains("must not be async"));
    }
}
