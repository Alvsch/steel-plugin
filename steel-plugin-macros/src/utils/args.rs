use syn::{
    LitInt, LitStr, Token,
    parse::{Parse, ParseBuffer, ParseStream},
};

#[derive(Debug)]
pub struct PluginMetaArgs {
    pub depends: Vec<String>,
}

impl Parse for PluginMetaArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut depends = vec![];

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
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

        Ok(PluginMetaArgs { depends })
    }
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use syn::parse_str;

    use super::{EventPriority, PluginMetaArgs};

    #[test]
    fn parses_depends_list() {
        let args: PluginMetaArgs =
            parse_str("depends = [\"a\", \"b\"]").expect("depends list should parse");
        assert_eq!(args.depends, vec!["a", "b"]);
    }

    #[test]
    fn rejects_unknown_plugin_meta_key() {
        let err = parse_str::<PluginMetaArgs>("unknown = []")
            .expect_err("unknown key should be rejected");
        assert!(err.to_string().contains("unknown key"));
    }

    #[test]
    fn parses_event_priority() {
        let priority: EventPriority = parse_str("priority = -1").expect("priority should parse");
        assert_eq!(priority.0, -1);
    }

    #[test]
    fn rejects_out_of_range_priority() {
        let err = parse_str::<EventPriority>("priority = 128")
            .expect_err("out of range priority should fail");
        assert!(err.to_string().contains("priority must be a valid i8"));
    }
}
