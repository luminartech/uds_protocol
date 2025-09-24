#![warn(clippy::pedantic)]
//! Blanket/Common types and traits for identifiers (Data Identifiers and Routine Identifiers)
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive Identifier and implement `TryFrom<u16>`, `Into<u16>` traits
///
/// ## Enum Example
/// ```rust
/// use uds_protocol::{UDSRoutineIdentifier, Identifier, Error};
///
/// #[derive(Clone, Copy, Identifier, Serialize)]
/// pub enum MyRoutineIdentifier {
///    /// 0x0101 (example)
///    VerifySignature,
///
///    // Standard ISO UDS routine fallthrough
///    UDSRoutineIdentifier(UDSRoutineIdentifier),
/// }
///
/// impl TryFrom<u16> for MyRoutineIdentifier {
///    type Error = uds_protocol::Error;
///    fn try_from(value: u16) -> Result<Self, Self::Error> {
///      match value {
///        0x0101 => Ok(MyRoutineIdentifier::VerifySignature),
///        _ => Ok(MyRoutineIdentifier::UDSRoutineIdentifier(UDSRoutineIdentifier::try_from(value)?)),
///      }
///   }
/// }
///
/// impl From<MyRoutineIdentifier> for u16 {
///    fn from(value: MyRoutineIdentifier) -> Self {
///      match value {
///        MyRoutineIdentifier::VerifySignature => 0x0101,
///        MyRoutineIdentifier::UDSRoutineIdentifier(identifier) => u16::from(identifier),
///      }
///    }
/// }
/// ```
///
/// ## Struct definition Example
/// Structs can only contain a single value to be used as an identifier to constrain the type
/// ```rust
///
/// use uds_protocol::{UDSIdentifier, Identifier};
///
/// #[derive(Clone, Copy, Identifier, Serialize)]
/// pub struct ProtocolIdentifier {
///    identifier: UDSIdentifier,
/// }
/// ```
///
/// # Panics
///
/// This will panic if `syn::Data::Union()` type is passed as input
///
#[proc_macro_derive(Identifier)]
pub fn uds_identifier_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Validate shape; on failure, return compile_error! tokens.
    if let Err(e) = validate_identifier_shape(&input) {
        return e.to_compile_error().into();
    }

    let name = &input.ident;
    let expanded = quote! {
        impl Identifier for #name {}
    };

    TokenStream::from(expanded)
}

fn validate_identifier_shape(input: &DeriveInput) -> Result<(), syn::Error> {
    match &input.data {
        // Accept any enum
        syn::Data::Enum(_) => Ok(()),

        // Sometimes we use a struct to simply pass through the identifier, accept those as well
        syn::Data::Struct(s) => {
            if let syn::Fields::Named(fields) = &s.fields {
                if fields.named.len() == 1 {
                    Ok(())
                } else {
                    Err(syn::Error::new_spanned(
                        &s.fields,
                        "Identifier can only be derived for structs with a single member",
                    ))
                }
            } else {
                Ok(())
            }
        }

        // Reject unions with a nice error (don’t panic)
        syn::Data::Union(u) => Err(syn::Error::new_spanned(
            u.union_token,
            "Identifier can only be derived for enums and structs",
        )),
    }
}
