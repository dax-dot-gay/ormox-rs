use darling::{ast::NestedMeta, FromField, FromMeta};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Ident, Type};

#[derive(FromMeta, Debug)]
pub(crate) struct DocumentMetadata {
    pub collection: String,

    #[darling(default)]
    pub id_field: Option<String>,

    #[darling(default)]
    pub id_alias: Option<String>
}

#[derive(FromField, Debug)]
#[darling(attributes(index))]
#[allow(dead_code)]
pub(crate) struct FieldIndex {
    pub ident: Option<syn::Ident>,
    pub ty: Type,

    #[darling(default)]
    pub unique: bool,

    #[darling(default)]
    pub name: Option<String>,

    #[darling(default)]
    pub alias: Option<String>
}

pub(crate) fn wrap_document(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = match syn::parse2::<syn::ItemStruct>(input) {
        Ok(is) => is,
        Err(e) => return darling::Error::from(e).write_errors()
    };
    let attr_args = match NestedMeta::parse_meta_list(args) {
        Ok(v) => v,
        Err(e) => return darling::Error::from(e).write_errors()
    };
    let args = match DocumentMetadata::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return darling::Error::from(e).write_errors()
    };

    let struct_name = &input.ident;
    let mut original_struct = input.clone();
    let mut index_objs: Punctuated<syn::ExprStruct, Comma> = Punctuated::new();
    let mut creation_fields = Punctuated::<syn::FnArg, Comma>::new();
    let mut creation_assignments = Punctuated::<syn::FieldValue, Comma>::new();
    let collection = args.collection;
    let id_field = args.id_field.unwrap_or("_docid".into());
    let id_alias = args.id_alias.unwrap_or(id_field.clone());
    let id_ident = Ident::new(&id_field.clone(), Span::call_site());


    match original_struct.fields {
        syn::Fields::Named(ref mut existing) => {
            for field in existing.named.clone() {
                if let Some(ident) = field.ident.clone() {
                    if ident.to_string() == id_field {
                        return quote! {compile_error!("Document ID fields are defined by the ORM.")};
                    }

                    if ident.to_string() == "_collection" {
                        return quote! {compile_error!("The _collection field is reserved for the ORM.")};
                    }

                    if field.attrs.iter().any(|a| a.path().segments.last().and_then(|s| Some(s.ident.to_string() == String::from("index"))).or(Some(false)).unwrap()) {
                        let field_index = match FieldIndex::from_field(&field) {
                            Ok(fi) => fi,
                            Err(e) => return darling::Error::from(e).write_errors()
                        };

                        let alias = field_index.alias.unwrap_or(field_index.ident.unwrap().to_string());
                        let name = field_index.name.unwrap_or(alias.clone());
                        let unique = field_index.unique;

                        index_objs.push(syn::parse_quote!{ormox::Index {fields: vec![String::from(#alias)], name: Some(String::from(#name)), unique: #unique}});
                    }

                    let ftype = field.ty.clone();

                    creation_fields.push(syn::parse_quote!{#ident: impl Into<#ftype>});
                    creation_assignments.push(syn::parse_quote!{#ident: #ident.into()});
                }
            }

            existing.named.push(syn::parse_quote!{
                #[serde(default = "ormox::ormox_core::uuid::Uuid::new_v4", rename = #id_alias)]
                #id_ident : ormox::ormox_core::uuid::Uuid
            });

            existing.named.push(syn::parse_quote!{
                #[serde(default, skip)]
                _collection: Option<ormox::ormox_core::client::Collection<Self>>
            });
        },
        syn::Fields::Unnamed(_) => return quote! {compile_error!("This macro only supports fields structs with named fields.")},
        syn::Fields::Unit => return quote! {compile_error!("This macro does not support unit structs.")}
    };

    quote! {
        #[derive(ormox::ormox_core::serde::Serialize, ormox::ormox_core::serde::Deserialize, Clone, ormox::Document)]
        #original_struct

        impl ormox::Document for #struct_name {
            fn id(&self) -> ormox::ormox_core::uuid::Uuid {
                self.#id_ident.clone()
            }

            fn id_field() -> String {
                String::from(#id_alias)
            }

            fn collection_name() -> String {
                String::from(#collection)
            }

            fn indexes() -> Vec<ormox::Index> {
                vec![#index_objs]
            }

            fn attached_collection(&self) -> Option<ormox::Collection<Self>> {
                self._collection.clone()
            }

            fn attach_collection(&mut self, collection: ormox::Collection<Self>) -> () {
                self._collection = Some(collection.clone());
            }
        }

        impl #struct_name {
            pub fn create(collection: Option<ormox::Collection<Self>>, #creation_fields) -> Self {
                Self {
                    #id_ident: ormox::ormox_core::uuid::Uuid::new_v4(),
                    _collection: collection.clone(),
                    #creation_assignments
                }
            }
        }
    }
}

