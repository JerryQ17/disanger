use macro_core::{add_default_field_name, pretty_named_struct};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Field, Fields, FieldsNamed, ItemStruct, Token};

pub fn impl_attributed_field(mut input: ItemStruct) -> TokenStream {
    let ident = &input.ident;
    match &mut input.fields {
        Fields::Unit => impl_attributed_field(ItemStruct {
            fields: Fields::Named(FieldsNamed {
                named: Punctuated::new(),
                brace_token: Default::default(),
            }),
            ..input
        }),
        Fields::Unnamed(unnamed) => {
            let output = ItemStruct {
                ident: ident.clone(),
                fields: add_default_field_name(unnamed),
                ..input
            }
            .into_token_stream();
            abort!(
                unnamed,
                "struct `{}` has unnamed fields `{}`", ident, unnamed.to_token_stream();
                note = "attributed_field can only be applied to structs with named fields";
                help = "add names to the fields:\n{}", pretty_named_struct(output);
            );
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let from_field = impl_from_field(ident, named);
            named.push(parse_quote!(__original: syn::Field));
            let extra_getters = impl_extra_getters(named);
            quote! {
                #input
                impl #ident {
                    #extra_getters
                }
                #from_field
            }
        }
    }
}

fn impl_extra_getters(fields: &Punctuated<Field, Token![,]>) -> TokenStream {
    let extra_fields = [
        (
            quote! { vis },
            quote! { &syn::Visibility },
            quote! { &self.__original.vis },
        ),
        (
            quote! { mutability },
            quote! { &syn::FieldMutability },
            quote! { &self.__original.mutability },
        ),
        (
            quote! { ident },
            quote! { Option<&syn::Ident> },
            quote! { self.__original.ident.as_ref() },
        ),
        (
            quote! { ty },
            quote! { &syn::Type },
            quote! { &self.__original.ty },
        ),
    ];
    let getters = extra_fields.into_iter().map(|(mut ident, ty, getter)| {
        fields
            .iter()
            .any(|field| *field.ident.as_ref().unwrap() == ident.to_string())
            .then(|| ident = format_ident!("__{ident}").to_token_stream());
        quote! { pub fn #ident(&self) -> #ty { #getter } }
    });
    quote! { #(#getters)* }
}

fn impl_from_field(name: &Ident, fields: &Punctuated<Field, Token![,]>) -> TokenStream {
    if fields.is_empty() {
        return quote! {
            impl From<syn::Field> for #name {
                fn from(field: syn::Field) -> Self {
                    Self { __original: field }
                }
            }
        };
    }
    let mut decl = vec![];
    let mut arms = vec![];
    let mut assign = vec![];
    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let ident_str = ident.to_string();
        decl.push(quote! { let mut #ident = #ty::default();});
        arms.push(quote! {
            #ident_str => {
                let evaluated = evalexpr::eval(&*tokens_str)
                    .unwrap_or_else(|e| proc_macro_error::abort!(
                        tokens,
                        "failed to evaluate `{}`: {}", tokens, e;
                        note = "the value of attribute `{}` is not a valid Rust expression", #ident_str;
                        )
                    );
                #ident = evaluated
                    .clone()
                    .try_into()
                    .unwrap_or_else(|e| proc_macro_error::abort!(
                        tokens,
                        "failed to convert `{}` to type `{}`: {}", tokens, stringify!(#ty), e;
                        note = "the evaluated value of attribute `{}` is `{}`", #ident_str, evaluated;
                        )
                    );
            }
        });
        assign.push(quote! { #ident });
    }
    let matches = quote! {
        match ident.as_str() {
            #(#arms)*
            _ => {}
        }
    };
    quote! {
        impl From<syn::Field> for #name {
            fn from(field: syn::Field) -> Self {
                #(#decl)*
                for attr in &field.attrs {
                    let (s, tokens) = match &attr.meta {
                        syn::Meta::Path(syn::Path { segments, .. }) => (segments, quote! { true }),
                        syn::Meta::List(syn::MetaList {
                            path: syn::Path { segments, .. },
                            tokens,
                            ..
                        }) => (segments, tokens.clone()),
                        syn::Meta::NameValue(syn::MetaNameValue {
                            path: syn::Path { segments, .. },
                            value,
                            ..
                        }) => (segments, value.to_token_stream()),
                    };
                    let ident = s.last().unwrap().ident.to_string();
                    let tokens_str = tokens.to_string();
                    #matches
                }
                Self { #(#assign,)* __original: field }
            }
        }
    }
}
