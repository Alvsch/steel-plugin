use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, FnArg, Ident, ItemFn, Visibility, parse_macro_input, spanned::Spanned};

use crate::args::EventHandlerArgs;
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

    if let Some(expected_name) = rules.name
        && sig.ident != expected_name
    {
        return Err(Error::new(
            sig.ident.span(),
            format!("function must be named '{expected_name}'"),
        ));
    }

    match rules.params {
        None => (),
        Some([]) => {
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
            name: Some("on_enable"),
            require_pub: true,
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
            name: Some("on_disable"),
            require_pub: true,
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
pub fn event_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    let args = parse_macro_input!(args as EventHandlerArgs);

    if let Err(err) = validate(
        &FnRules {
            require_pub: true,
            ret: Some("EventResult"),
            ..Default::default()
        },
        &item,
    ) {
        return err.to_compile_error().into();
    }

    let priority = args.priority;
    let flags = args.flags.unwrap_or_else(
        || syn::parse_quote! { ::steel_plugin_sdk::event::EventHandlerFlags::empty() },
    );

    let export_fn_name = format_ident!("__{}", item.sig.ident);
    let export_fn_name_str = export_fn_name.to_string();
    let impl_fn_name = format_ident!("{}_impl", export_fn_name);
    let handler_const = format_ident!("__{}", item.sig.ident.to_string().to_uppercase());
    let stmts = &item.block.stmts;

    let inputs = &item.sig.inputs;
    let (event_pat, event_ty) = if let Some(FnArg::Typed(pat_ty)) = inputs.first() {
        (&pat_ty.pat, &pat_ty.ty)
    } else {
        return Error::new(item.sig.span(), "expected at least one parameter")
            .into_compile_error()
            .into();
    };

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #export_fn_name(ptr: u32, len: u32) -> u64 {
            fn #impl_fn_name(#event_pat: #event_ty) -> EventResult {
                #(#stmts)*
            }

            let event = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
            let event: #event_ty = rmp_serde::from_slice(event).unwrap();
            let result = #impl_fn_name(event);
            result.as_u64()
        }

        pub const #handler_const: ::steel_plugin_sdk::event::handler::EventHandler = ::steel_plugin_sdk::event::handler::EventHandler {
            event_name: std::borrow::Cow::Borrowed(<#event_ty as ::steel_plugin_sdk::event::Event>::NAME),
            handler_name: std::borrow::Cow::Borrowed(#export_fn_name_str),
            priority: #priority,
            flags: #flags,
        };
    }
    .into()
}

#[proc_macro]
pub fn register_event(input: TokenStream) -> TokenStream {
    let fn_name = parse_macro_input!(input as Ident);
    let handler_const = format_ident!("__{}", fn_name.to_string().to_uppercase());
    quote! {
        ::steel_plugin_sdk::register_handler(&#handler_const);
    }
    .into()
}
