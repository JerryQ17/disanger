use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

mod adb_socket_family;

/// Derive the `AdbSocketFamily` trait for a struct or enum.
///
/// For structs, the trait generates:
/// - [`std::fmt::Display`] implementation.
/// - [`std::str::FromStr`] implementation.
/// - [`adb::socket::AdbSocketFamily`] implementation.
/// For enums, the trait generates:
/// - [`From`] implementations for each variant.
/// - [`std::fmt::Display`] implementation. (calls variant's `Display` implementation)
/// - [`std::str::FromStr`] implementation. (calls variant's `FromStr` implementation)
/// - [`adb::socket::AdbSocketFamily`] implementation.
#[proc_macro_error]
#[proc_macro_derive(AdbSocketFamily)]
pub fn derive_adb_socket_family(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    adb_socket_family::impl_adb_socket_family(input).into()
}
