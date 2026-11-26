use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub(crate) fn sdk_export_path() -> TokenStream {
    match crate_name("steel-plugin-sdk").expect("steel-plugin-sdk not found in Cargo.toml") {
        FoundCrate::Itself => quote!(crate::__export),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident::__export)
        }
    }
}
