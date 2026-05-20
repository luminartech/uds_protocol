#![warn(clippy::pedantic)]
//! Blanket/Common types and traits for identifiers (Data Identifiers and Routine Identifiers)
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive Identifier and implement `TryFrom<u16>`, `Into<u16>` traits
///
/// The derive itself emits only `impl Identifier for #name {}` — it
/// asserts trait conformance and nothing else. Callers supply their
/// own `TryFrom<u16>` / `From<#name> for u16` so the wire-format
/// dispatch matches the application's own identifier-space layout.
/// The examples below show that hand-written pattern alongside the
/// derive.
///
/// ## Enum Example
///
/// Adds a custom `VerifySignature` routine identifier on top of the
/// standard ISO UDS set via a fallthrough variant. The
/// `UDSRoutineIdentifier` type has a total `From<u16>` (every u16
/// maps to one of its variants, including reserved ranges), so the
/// fallthrough arm uses the infallible `::from` rather than `?`.
///
/// ```rust
/// use uds_protocol::{Error, Identifier, UDSRoutineIdentifier};
///
/// #[derive(Clone, Copy, Identifier)]
/// pub enum MyRoutineIdentifier {
///     /// 0x0101 (example)
///     VerifySignature,
///
///     /// Standard ISO UDS routine fallthrough.
///     UDSRoutineIdentifier(UDSRoutineIdentifier),
/// }
///
/// impl TryFrom<u16> for MyRoutineIdentifier {
///     type Error = Error;
///     fn try_from(value: u16) -> Result<Self, Self::Error> {
///         match value {
///             0x0101 => Ok(MyRoutineIdentifier::VerifySignature),
///             // `UDSRoutineIdentifier::from` is total over u16,
///             // so the fallthrough never fails.
///             _ => Ok(MyRoutineIdentifier::UDSRoutineIdentifier(
///                 UDSRoutineIdentifier::from(value),
///             )),
///         }
///     }
/// }
///
/// impl From<MyRoutineIdentifier> for u16 {
///     fn from(value: MyRoutineIdentifier) -> Self {
///         match value {
///             MyRoutineIdentifier::VerifySignature => 0x0101,
///             MyRoutineIdentifier::UDSRoutineIdentifier(identifier) => u16::from(identifier),
///         }
///     }
/// }
/// ```
///
/// ## Struct definition Example
///
/// Structs can only contain a single value to be used as an identifier
/// to constrain the type. `UDSIdentifier` has a real `TryFrom<u16>`
/// (returning `uds_protocol::Error` for out-of-range values), so the
/// struct wrapper's own `TryFrom` propagates via `?`.
///
/// ```rust
/// use uds_protocol::{Error, Identifier, UDSIdentifier};
///
/// #[derive(Clone, Copy, Identifier)]
/// pub struct WrappedIdentifier {
///     identifier: UDSIdentifier,
/// }
///
/// impl TryFrom<u16> for WrappedIdentifier {
///     type Error = Error;
///     fn try_from(value: u16) -> Result<Self, Self::Error> {
///         Ok(WrappedIdentifier {
///             identifier: UDSIdentifier::try_from(value)?,
///         })
///     }
/// }
///
/// impl From<WrappedIdentifier> for u16 {
///     fn from(value: WrappedIdentifier) -> Self {
///         u16::from(value.identifier)
///     }
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
