use syn::{
    Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct PluginExportInput {
    pub plugin: Ident,
    pub meta: Option<PluginMetaInput>,
}

#[derive(Debug, Default)]
pub struct PluginMetaInput {
    pub depends: Vec<String>,
}

impl Parse for PluginExportInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let plugin_type: Ident = input.parse()?;

        let meta = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                None
            } else {
                let content;
                syn::braced!(content in input);
                Some(content.parse::<PluginMetaInput>()?)
            }
        } else {
            None
        };

        Ok(PluginExportInput {
            plugin: plugin_type,
            meta,
        })
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
                    depends = items.into_iter().map(|s| s.value()).collect();
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

        Ok(PluginMetaInput { depends })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn parses_depends_list() {
        let meta: PluginMetaInput = parse_str("depends: [\"a\", \"b\"]").unwrap();
        assert_eq!(meta.depends, vec!["a", "b"]);
    }

    #[test]
    fn parses_empty_depends_list() {
        let meta: PluginMetaInput = parse_str("depends: []").unwrap();
        assert!(meta.depends.is_empty());
    }

    #[test]
    fn parses_single_dependency() {
        let meta: PluginMetaInput = parse_str("depends: [\"logger\"]").unwrap();
        assert_eq!(meta.depends, vec!["logger"]);
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
        let err = parse_str::<PluginMetaInput>("unknown: []").unwrap_err();
        assert!(err.to_string().contains("unknown metadata field `unknown`"));
    }

    #[test]
    fn rejects_missing_colon() {
        assert!(parse_str::<PluginMetaInput>("depends [\"a\"]").is_err());
    }

    #[test]
    fn rejects_non_string_in_depends() {
        assert!(parse_str::<PluginMetaInput>("depends: [123]").is_err());
    }

    #[test]
    fn parses_bare_export() {
        let input: PluginExportInput = parse_str("ConsumerPlugin").unwrap();
        assert_eq!(input.plugin.to_string(), "ConsumerPlugin");
        assert!(input.meta.is_none());
    }

    #[test]
    fn parses_export_with_empty_meta_block() {
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

    #[test]
    fn parses_export_with_trailing_comma_after_meta() {
        let input: PluginExportInput =
            parse_str("ConsumerPlugin, { depends: [\"logger\",] }").unwrap();
        assert_eq!(input.meta.unwrap().depends, vec!["logger"]);
    }
}
