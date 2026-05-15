use syn::{
    LitInt, Token,
    parse::{Parse, ParseStream},
};

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

    use super::EventPriority;

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
