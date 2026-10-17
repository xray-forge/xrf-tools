#![doc = include_str!("../README.md")]

mod constant;

use proc_macro::TokenStream;
use syn::Error;

/// Exports a `const` item to the frontend's generated bindings, registering its name, doc comment and value; see the
/// crate's README.
#[proc_macro_attribute]
pub fn constant(arguments: TokenStream, item: TokenStream) -> TokenStream {
  constant::expand(arguments.into(), item.into())
    .unwrap_or_else(Error::into_compile_error)
    .into()
}
