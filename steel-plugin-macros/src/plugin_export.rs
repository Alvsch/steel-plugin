use std::env;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use semver::Version;
use steel_plugin_core::PluginMeta;
use syn::{
    Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::sdk_path::sdk_export_path;

pub(crate) struct PluginExportInput {
    plugin: Ident,
    meta: Option<PluginMetaInput>,
}

#[derive(Debug, Default)]
struct PluginMetaInput {
    depends: Vec<String>,
}

impl Parse for PluginExportInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let plugin = input.parse()?;

        let meta = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                None
            } else {
                let content;
                syn::braced!(content in input);
                Some(content.parse()?)
            }
        } else {
            None
        };

        Ok(Self { plugin, meta })
    }
}

impl Parse for PluginMetaInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut depends = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "depends" => {
                    let content;
                    syn::bracketed!(content in input);
                    let items: Punctuated<LitStr, Token![,]> =
                        Punctuated::parse_terminated(&content)?;
                    depends = items.into_iter().map(|value| value.value()).collect();
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown metadata field `{other}`"),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { depends })
    }
}

pub(crate) fn expand(PluginExportInput { plugin, meta }: PluginExportInput) -> TokenStream {
    let meta = meta.unwrap_or_default();

    let meta = PluginMeta {
        name: env::var("CARGO_PKG_NAME").expect("no name"),
        description: env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default(),
        version: Version::parse(&env::var("CARGO_PKG_VERSION").unwrap_or_default())
            .expect("invalid version"),
        authors: env::var("CARGO_PKG_AUTHORS")
            .map_or_default(|authors| authors.split(':').map(ToString::to_string).collect()),
        depends: meta.depends,
        api_version: steel_plugin_core::STEEL_API_VERSION,
    };

    if meta.name == "steel" {
        return syn::Error::new(Span::call_site(), "The plugin name 'steel' is reserved")
            .to_compile_error();
    }

    let bytes = meta.serialize();
    let len = bytes.len();
    let export = sdk_export_path();

    quote! {
        const _: () = {
            #[unsafe(link_section = "steel-api::plugin::metadata")]
            #[used]
            static __PLUGIN_METADATA: [u8; #len] = [#(#bytes),*];

            #export::export!(#plugin);
        };
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{PluginExportInput, PluginMetaInput};
    use syn::parse_str;

    #[test]
    fn parses_depends_list() {
        let meta: PluginMetaInput = parse_str("depends: [\"a\", \"b\"]").unwrap();
        assert_eq!(meta.depends, vec!["a", "b"]);
    }

    #[test]
    fn parses_single_dependency() {
        let meta: PluginMetaInput = parse_str("depends: [\"logger\"]").unwrap();
        assert_eq!(meta.depends, vec!["logger"]);
    }

    #[test]
    fn parses_empty_depends_list() {
        let meta: PluginMetaInput = parse_str("depends: []").unwrap();
        assert_eq!(meta.depends, [] as [String; 0]);
    }

    #[test]
    fn parses_depends_with_trailing_comma() {
        let meta: PluginMetaInput = parse_str("depends: [\"a\", \"b\",]").unwrap();
        assert_eq!(meta.depends, vec!["a", "b"]);
    }

    #[test]
    fn parses_empty_meta_block() {
        let meta: PluginMetaInput = parse_str("").unwrap();
        assert!(meta.depends.is_empty());
    }

    #[test]
    fn rejects_unknown_key() {
        let error = parse_str::<PluginMetaInput>("unknown: []").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unknown metadata field `unknown`")
        );
    }

    #[test]
    fn rejects_missing_colon() {
        assert!(parse_str::<PluginMetaInput>("depends [\"a\"]").is_err());
    }

    #[test]
    fn rejects_non_string_dependency() {
        assert!(parse_str::<PluginMetaInput>("depends: [123]").is_err());
    }

    #[test]
    fn parses_bare_export() {
        let input: PluginExportInput = parse_str("ConsumerPlugin").unwrap();
        assert_eq!(input.plugin.to_string(), "ConsumerPlugin");
        assert!(input.meta.is_none());
    }

    #[test]
    fn parses_export_with_empty_metadata() {
        let input: PluginExportInput = parse_str("ConsumerPlugin, {}").unwrap();
        assert_eq!(input.plugin.to_string(), "ConsumerPlugin");
        assert!(input.meta.is_some());
    }

    #[test]
    fn parses_export_with_depends() {
        let input: PluginExportInput =
            parse_str("ConsumerPlugin, { depends: [\"logger\", \"event-bus\"] }").unwrap();
        assert_eq!(input.plugin.to_string(), "ConsumerPlugin");
        assert_eq!(input.meta.unwrap().depends, vec!["logger", "event-bus"]);
    }
}
