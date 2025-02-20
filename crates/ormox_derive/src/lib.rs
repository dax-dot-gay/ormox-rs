mod document;
use quote::quote;

#[proc_macro_attribute]
pub fn ormox_document(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    document::wrap_document(args.into(), input.into()).into()
}

#[proc_macro_derive(Document, attributes(index))]
pub fn derive_document_helper(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {}.into()
}