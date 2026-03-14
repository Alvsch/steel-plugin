use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemFn, Visibility};

pub struct FnRules<'a> {
    pub name: Option<&'a str>,
    pub params: Option<&'a [&'a str]>,
    pub ret: Option<&'a str>,
    pub require_pub: bool,
}

impl Default for FnRules<'_> {
    fn default() -> Self {
        Self {
            name: None,
            params: None,
            ret: None,
            require_pub: true,
        }
    }
}

pub(crate) fn validate(rules: &FnRules, item: &ItemFn) -> Result<(), Error> {
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
