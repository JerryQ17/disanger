use proc_macro2::{Ident, TokenStream};
use syn::spanned::Spanned;
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed};

/// Convert unnamed fields to named fields with default names `field0`, `field1`, etc.
pub fn add_default_field_name(unnamed: &FieldsUnnamed) -> Fields {
    Fields::Named(FieldsNamed {
        named: unnamed
            .unnamed
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, f)| Field {
                ident: Some(Ident::new(&format!("field{}", i), f.span())),
                ..f
            })
            .collect(),
        brace_token: Default::default(),
    })
}

/// Display a struct with named fields in a pretty format.
pub fn pretty_named_struct(input: TokenStream) -> String {
    input
        .to_string()
        .replace("{ ", "{\n\t")
        .replace(" }", "\n}")
        .replace(", ", ",\n\t")
        .replace(" :", ":")
}
