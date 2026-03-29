use rmp::encode::{ValueWriteError, write_array_len, write_map_len, write_str, write_u32};
use syn::{
    LitInt, LitStr, Token,
    parse::{Parse, ParseBuffer, ParseStream},
};

#[derive(Debug)]
pub struct PluginMetaArgs {
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub depends: Vec<String>,
}

impl PluginMetaArgs {
    pub fn serialize(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_map_len(&mut buf, 4)?;

        write_str(&mut buf, "name")?;
        write_str(&mut buf, &self.name)?;

        write_str(&mut buf, "version")?;
        write_str(&mut buf, &self.version)?;

        write_str(&mut buf, "api_version")?;
        write_u32(&mut buf, self.api_version)?;

        write_str(&mut buf, "depends")?;
        write_array_len(&mut buf, self.depends.len() as u32)?;
        for dep in &self.depends {
            write_str(&mut buf, dep)?;
        }

        Ok(buf)
    }
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
