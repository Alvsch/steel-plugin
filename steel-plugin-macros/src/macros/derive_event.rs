use proc_macro2::TokenStream;
use quote::quote;
use steel_plugin_core::fnv1a_32;
use syn::DeriveInput;

pub(crate) fn derive_event(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let type_name = ident.to_string();
    let topic_id = fnv1a_32(type_name.as_bytes());

    quote! {
        impl #impl_generics crate::event::Event for #ident #ty_generics #where_clause {
            const TOPIC_ID: ::steel_plugin_core::TopicId = #topic_id;
        }
    }
}
