use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;

#[derive(FromDeriveInput, Default)]
#[darling(attributes(ormox), forward_attrs(allow, doc, cfg))]
pub struct DocumentOpts {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    collection: String
}

pub(crate) fn derive_doc(input: TokenStream) -> TokenStream {
    
}

