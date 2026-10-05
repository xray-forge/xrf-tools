#![doc = include_str!("../README.md")]

mod shader_struct;

use proc_macro::TokenStream;
use syn::{DeriveInput, Error, parse_macro_input};

/// Lays a `#[repr(C)]` struct out as WGSL does and asserts, at compile time, that Rust laid it out the same; see the
/// crate's README.
#[proc_macro_derive(ShaderStruct)]
pub fn shader_struct_derive(input: TokenStream) -> TokenStream {
  let input: DeriveInput = parse_macro_input!(input as DeriveInput);

  shader_struct::expand(&input)
    .unwrap_or_else(Error::into_compile_error)
    .into()
}
