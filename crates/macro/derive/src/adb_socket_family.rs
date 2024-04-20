use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, abort_if_dirty, emit_error};
use quote::{format_ident, quote, ToTokens};
use syn::{Data, DeriveInput, Fields, Index};

use macro_core_impl::attributed_field;

attributed_field! { struct AdbSocketFamilyField; }

pub fn impl_adb_socket_family(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    match input.data {
        Data::Struct(ds) => {
            let fields: Vec<AdbSocketFamilyField> = match ds.fields {
                Fields::Named(named) => named.named,
                Fields::Unnamed(unnamed) => unnamed.unnamed,
                Fields::Unit => abort!(
                    ds.fields, "`AdbSocketFamily` can only be derived for structs with fields";
                    note = "`{}` has no fields", input.ident;
                    help = "add fields to the struct";
                ),
            }
            .into_iter()
            .map(|f| f.into())
            .collect();
            let family = ident.to_string().to_lowercase();
            let display = impl_display(&family, &input.ident, &fields);
            let from_str = impl_from_str(&family, &input.ident, &fields);
            quote! {
                #display
                #from_str
                impl AdbSocketFamily for #ident {}
            }
        }
        Data::Enum(de) => {
            let mut from_variants = Vec::new();
            let mut display_arms = Vec::new();
            let mut from_str_arms = Vec::new();
            for variant in de.variants {
                let variant_ident = &variant.ident;
                let fields = match variant.fields {
                    Fields::Named(named) => named.named,
                    Fields::Unnamed(unnamed) => unnamed.unnamed,
                    Fields::Unit => abort!(
                        variant.fields, "`AdbSocketFamily` can only be derived for structs with fields";
                        note = "`{}` has no fields", variant_ident;
                        help = "add fields to the struct";
                    ),
                };
                if fields.len() > 1 {
                    emit_error!(
                        fields, "`AdbSocketFamily` can only be derived for structs with one field";
                        note = "`{}` has multiple fields", variant_ident;
                        help = "remove fields from the struct";
                    );
                }
                let field = fields.first().unwrap();
                let field_ty = &field.ty;
                from_variants.push(quote! {
                    impl From<#field_ty> for #ident {
                            fn from(value: #field_ty) -> Self {
                                Self::#variant_ident(value)
                        }
                    }
                });
                display_arms.push(quote! {
                    Self::#variant_ident(value) => write!(f, "{}", value),
                });
                from_str_arms.push(quote! {
                    if let Ok(value) = s.parse() {
                        return Ok(Self::#variant_ident(value));
                    }
                });
            }
            abort_if_dirty();
            quote! {
                #(#from_variants)*
                impl std::fmt::Display for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#display_arms)*
                        }
                    }
                }
                impl std::str::FromStr for #ident {
                    type Err = crate::error::AdbError;
                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        #(#from_str_arms)*
                        Err(crate::error::AdbError::Parse {
                            value: s.to_string(),
                            source_type: "&str",
                            target_type: stringify!(#ident),
                            source: None,
                        })
                    }
                }
                impl AdbSocketFamily for #ident {}
            }
        }
        Data::Union(_) => abort!(
            input, "`AdbSocketFamily` can only be derived for structs";
            note = "`{}` is a union, not a struct", input.ident;
        ),
    }
}

fn impl_display(family: &str, ident: &Ident, fields: &[AdbSocketFamilyField]) -> TokenStream {
    let mut format = format!("{}:", family);
    let fields = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            if i > 0 {
                format.push(':');
            }
            format.push_str("{}");
            let ident = f
                .ident()
                .map(Ident::to_token_stream)
                .unwrap_or_else(|| Index::from(i).to_token_stream());
            if f.ty().to_token_stream().to_string() == "PathBuf" {
                quote! { #ident.display() }
            } else {
                ident
            }
        })
        .collect::<Vec<_>>();
    quote! {
        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #format #(, self.#fields)*)
            }
        }
    }
}

fn err<T: ToTokens>(ident: &str, ty: &T, source: bool) -> TokenStream {
    let ident = format_ident!("{}", ident);
    let source = if source {
        quote! { source: Some(Box::new(e)), }
    } else {
        quote! { source: None, }
    };
    quote! {
        crate::error::AdbError::Parse {
            value: #ident.to_string(),
            source_type: "&str",
            target_type: stringify!(#ty),
            #source
        }
    }
}

fn impl_from_str(family: &str, ident: &Ident, fields: &[AdbSocketFamilyField]) -> TokenStream {
    let fields_count = fields.len();
    let mut decls = Vec::with_capacity(fields_count);
    let mut args = Vec::with_capacity(fields_count);
    for (i, f) in fields.iter().enumerate() {
        let f_ident = f
            .ident()
            .cloned()
            .unwrap_or_else(|| format_ident!("field{}", i));
        let some = err("rest", f.ty(), true);
        decls.push(if i < fields_count - 1 {
            let none = err("rest", f.ty(), false);
            quote! {
                let (#f_ident, rest) = match rest.split_once(':') {
                    Some((value, rest)) => (value.parse().map_err(|e| #some)?, rest),
                    None => return Err(#none),
                };
            }
        } else {
            quote! { let #f_ident = rest.parse().map_err(|e| #some)?; }
        });
        args.push(f_ident);
    }
    let new = if fields.first().unwrap().ident().is_some() {
        quote! { {#(#args),*}}
    } else {
        quote! { (#(#args),*) }
    };
    let none = err("s", ident, false);
    quote! {
        impl std::str::FromStr for #ident {
            type Err = crate::error::AdbError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.split_once(':') {
                    Some((#family, rest)) => {
                        #(#decls)*
                        Ok(Self #new)
                    }
                    _ => Err(#none),
                }
            }
        }
    }
}
