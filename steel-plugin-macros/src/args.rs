use rmp::encode::{write_array_len, write_map_len, write_str, write_u32};
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
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        write_map_len(&mut buf, 4).unwrap();

        write_str(&mut buf, "name").unwrap();
        write_str(&mut buf, &self.name).unwrap();

        write_str(&mut buf, "version").unwrap();
        write_str(&mut buf, &self.version).unwrap();

        write_str(&mut buf, "api_version").unwrap();
        write_u32(&mut buf, self.api_version).unwrap();

        write_str(&mut buf, "depends").unwrap();
        write_array_len(&mut buf, self.depends.len() as u32).unwrap();
        for dep in &self.depends {
            write_str(&mut buf, dep).unwrap();
        }

        buf
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
