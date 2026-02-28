use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, FnArg, ItemFn, Visibility, parse_macro_input, spanned::Spanned};

use crate::{args::PluginMetaArgs, rules::FnRules};

mod args;
mod rules;

#[proc_macro]
pub fn plugin_meta(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as PluginMetaArgs);

    let bytes: Vec<u8> = args.serialize();
    let len = bytes.len();

    quote! {
        #[unsafe(link_section = "plugin_meta")]
        #[used]
        pub static __PLUGIN_META_SECTION: [u8; #len] = [#(#bytes),*];

        #[unsafe(no_mangle)]
        pub extern "C" fn alloc(len: u32) -> u32 {
            let layout = std::alloc::Layout::from_size_align(len as usize, 1).unwrap();
            unsafe { std::alloc::alloc(layout) as u32 }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn dealloc(ptr: u32, len: u32) {
            let layout = std::alloc::Layout::from_size_align(len as usize, 1).unwrap();
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }
    }
    .into()
}

fn validate(rules: &FnRules, item: &ItemFn) -> Result<(), Error> {
    let sig = &item.sig;

    if sig.ident != rules.name {
        return Err(Error::new(
            sig.ident.span(),
            format!("function must be named '{}'", rules.name),
        ));
    }

    match rules.params {
        None => {
            if let Some(arg) = sig.inputs.first() {
                return Err(Error::new(arg.span(), "function must have no parameters"));
            }
        }
        Some(expected_types) => {
            if sig.inputs.len() != expected_types.len() {
                return Err(Error::new(
                    sig.inputs.span(),
                    format!(
                        "function must have exactly {} parameter(s)",
                        expected_types.len()
                    ),
                ));
            }
            for (arg, expected) in sig.inputs.iter().zip(expected_types) {
                let arg_ty = match arg {
                    FnArg::Typed(pat_ty) => {
                        let ty = &pat_ty.ty;
                        quote!(#ty).to_string()
                    }
                    FnArg::Receiver(r) => {
                        return Err(Error::new(
                            r.span(),
                            "function must not have a self parameter",
                        ));
                    }
                };
                if &arg_ty != expected {
                    return Err(Error::new(
                        arg.span(),
                        format!("expected parameter type '{expected}', found '{arg_ty}'"),
                    ));
                }
            }
        }
    }

    match rules.ret {
        None => {
            if let syn::ReturnType::Type(_, ty) = &sig.output {
                return Err(Error::new(ty.span(), "function must have no return type"));
            }
        }
        Some(expected_ret) => match &sig.output {
            syn::ReturnType::Default => {
                return Err(Error::new(
                    sig.span(),
                    format!("function must return '{expected_ret}'"),
                ));
            }
            syn::ReturnType::Type(_, ty) => {
                if quote!(#ty).to_string() != expected_ret {
                    return Err(Error::new(
                        ty.span(),
                        format!(
                            "expected return type '{expected_ret}', found '{}'",
                            quote!(#ty)
                        ),
                    ));
                }
            }
        },
    }

    if let Some(asyncness) = sig.asyncness.as_ref() {
        return Err(Error::new(asyncness.span(), "function must not be async"));
    }

    if rules.require_pub && !matches!(item.vis, Visibility::Public(_)) {
        return Err(Error::new(item.vis.span(), "function must be public"));
    }

    Ok(())
}

#[proc_macro_attribute]
pub fn on_enable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate(
        &FnRules {
            name: "on_enable",
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let stmts = &item.block.stmts;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_enable() {
            #(#stmts)*
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn on_disable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate(
        &FnRules {
            name: "on_disable",
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let stmts = &item.block.stmts;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_disable() {
            #(#stmts)*
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn on_event(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    if let Err(err) = validate(
        &FnRules {
            name: "on_event",
            params: Some(&["& [u8]"]),
            ret: Some("EventResult"),
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let mut inputs = item.sig.inputs.iter();
    let name1 = if let Some(FnArg::Typed(pat_ty)) = inputs.next() {
        let pat = &pat_ty.pat;
        quote! { #pat }
    } else {
        quote! { _arg0 }
    };

    let stmts = &item.block.stmts;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn on_event(ptr: u32, len: u32) -> u64 {
            fn on_event_impl(#name1: &[u8]) -> EventResult {
                #(#stmts)*
            }

            let event = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
            let result = on_event_impl(event);
            result.pack()
        }

    }
    .into()
}
