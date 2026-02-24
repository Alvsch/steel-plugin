use proc_macro::TokenStream;
use quote::quote;
use serde::Serialize;
use syn::{
    LitInt, LitStr, Token,
    parse::{Parse, ParseBuffer, ParseStream},
    parse_macro_input,
};

#[derive(Debug, Serialize)]
struct PluginMetaArgs {
    name: String,
    version: String,
    api_version: u32,
    depends: Vec<String>,
}

impl Parse for PluginMetaArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut api_version = None;
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
                "api_version" => {
                    api_version = Some(input.parse::<LitInt>()?.base10_parse::<u32>()?);
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

        Ok(PluginMetaArgs {
            name: name.ok_or_else(|| input.error("missing `name`"))?,
            version: version.ok_or_else(|| input.error("missing `version`"))?,
            api_version: api_version.ok_or_else(|| input.error("missing `api_version`"))?,
            depends,
        })
    }
}

#[proc_macro]
pub fn plugin_meta(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as PluginMetaArgs);

    let bytes: Vec<u8> = rmp_serde::to_vec_named(&args).unwrap();
    let len = bytes.len();

    quote! {
        #[unsafe(link_section = "plugin_meta")]
        #[used]
        pub static __PLUGIN_META_SECTION: [u8; #len] = [#(#bytes),*];
    }
    .into()
}
