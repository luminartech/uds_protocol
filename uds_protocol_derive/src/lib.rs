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
/// use serde::Serialize;
///
/// #[derive(Copy, Clone, Serialize, Identifier)]
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
/// use serde::Serialize;
///
/// #[derive(Clone, Copy, Serialize, Identifier)]
/// pub struct ProtocolIdentifier {
///    identifier: UDSIdentifier,
/// }
/// ```
#[proc_macro_derive(Identifier)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::manual_assert)]
pub fn uds_identifier_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        // Accept any enum
        syn::Data::Enum(_) => (),
        // Sometimes we use a struct to simply pass through the identifier, accept those as well
        syn::Data::Struct(s) => {
            if let syn::Fields::Named(fields) = s.fields {
                assert!(
                    fields.named.len() == 1,
                    "Identifier can only be derived for structs with a single member"
                );
            }
        }
        syn::Data::Union(_) => panic!("Identifier can only be derived for enums and structs"),
    }
    let name = &input.ident;
    let expanded = quote! {
        impl Identifier for #name {}
    };

    TokenStream::from(expanded)
}
