#![doc = include_str!("../README.md")]

mod pass_parameters;
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

/// Declares one bind group of a pass: its layout, its WGSL declarations, its graph accesses and its bindings, a binding
/// per field in order; see the crate's README.
#[proc_macro_derive(
  PassParameters,
  attributes(parameters, uniform, storage, texture, storage_texture, sampler)
)]
pub fn pass_parameters_derive(input: TokenStream) -> TokenStream {
  let input: DeriveInput = parse_macro_input!(input as DeriveInput);

  pass_parameters::expand(&input)
    .unwrap_or_else(Error::into_compile_error)
    .into()
}
