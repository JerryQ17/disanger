use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::ItemStruct;

mod attributed_field;

/// Generate field metadata getters to a struct with named or no fields .
///
/// This macro will generate the following code:
///
/// - Add a field `__original: syn::Field` representing the original field.
///   If the struct has no fields, the output struct will have a single field `__original`.
/// - Generate getters for the following metadata:
///     - `pub fn vis(&self) -> syn::Visibility`
///     - `pub fn mutability(&self) -> syn::FieldMutability`
///     - `pub fn ident(&self) -> syn::Ident`
///     - `pub fn ty(&self) -> syn::Type`
/// - Implement `From<syn::Field>` for the struct.
///
/// # Note
///
/// - The struct must have named or no fields.
/// - The struct **must not** have a field named `__original`.
/// - If the struct has fields named `vis`, `mutability`, `ident`, `ty`,
///   the metadata (not the field) getters will be generated with a prefix `__`.
///   For example, the getter for the metadata `vis` will be named `__vis`.
///
/// ```ignore
/// attributed_field! {
///     struct ConflictingField {
///         vis: T,     // conflict with the generated getter
///     }
/// }
///
/// // The generated struct will have the following getters:
///
/// impl ConflictingField {
///     pub fn __vis(&self) -> &syn::Visibility;    // getter for the metadata `vis`
///     // other getters omitted
/// }
/// ```
///
/// # Example
///
/// Assuming we are writing a derive macro for `TraitA`
/// with some derive macro helper attributes `helper1`, `helper2`:
///
/// ```ignore
/// #[derive(TraitA)]
/// struct Foo {
///     #[helper1]
///     a: i32,
///     #[helper2]
///     b: String,
/// }
///
/// #[proc_macro_derive(TraitA, attributes(helper1, helper2))]
/// fn derive_trait_a(input: TokenStream) -> TokenStream;
/// ```
///
/// We may define a helper struct `TraitAField` to save the metadata:
///
/// ```ignore
/// struct TraitAField {
///     ident: syn::Ident,
///     ty: syn::Type,
///     helper1: bool,
///     helper2: bool,
/// }
///
/// impl From<syn::Field> for TraitAField;  // implementation details omitted
/// ```
///
/// It's tedious to write the `From<syn::Field>` implementation manually.
/// We can use `attributed_field` macro to generate the implementation:
///
/// ```ignore
/// attributed_field! {
///     struct TraitAField {
///         helper1: bool,
///         helper2: bool,
///     }
/// }
/// ```
///
/// The `attributed_field` macro will generate the following code:
/// ```ignore
/// struct TraitAField {
///     __original: syn::Field,
///     helper1: bool,
///     helper2: bool,
/// }
///
/// impl TraitAField {
///     pub fn vis(&self) -> syn::Visibility;
///     pub fn mutability(&self) -> syn::FieldMutability;
///     pub fn ident(&self) -> Option<syn::Ident>;
///     pub fn ty(&self) -> syn::Type;
/// }
///
/// impl From<syn::Field> for TraitAField {
///     fn from(__original: syn::Field) -> Self;
/// }
///```
#[proc_macro_error]
#[proc_macro]
pub fn attributed_field(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemStruct);
    attributed_field::impl_attributed_field(input).into()
}
