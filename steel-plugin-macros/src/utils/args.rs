use steel_plugin_core::PluginMeta;
use syn::{
    LitInt, LitStr, Token,
    parse::{Parse, ParseBuffer, ParseStream},
};

#[derive(Debug)]
pub struct PluginMetaArgs(pub PluginMeta);

impl Parse for PluginMetaArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut depends = vec![];

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    name = Some(input.parse::<LitStr>()?.value());
                }
                "version" => {
                    version = Some(input.parse::<LitStr>()?.value());
                }
                "depends" => {
                    let content;
                    syn::bracketed!(content in input);
                    let deps = content.parse_terminated(ParseBuffer::parse, Token![,])?;
                    depends = deps.iter().map(LitStr::value).collect();
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown key `{other}`"),
                    ));
                }
            }

            // consume optional trailing comma
            let _ = input.parse::<Token![,]>();
        }

        Ok(PluginMetaArgs(PluginMeta {
            name: name.ok_or_else(|| input.error("missing `name`"))?,
            version: version.ok_or_else(|| input.error("missing `version`"))?,
            depends,
        }))
    }
}

pub struct EventPriority(pub i8);

impl Parse for EventPriority {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
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
