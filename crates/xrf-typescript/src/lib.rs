#![doc = include_str!("../README.md")]

mod symbol_resolver;
#[cfg(feature = "codegen")]
mod typescript_item_renderer;
mod typescript_source;
mod typescript_type_renderer;

pub use swc_common;
pub use swc_ecma_ast;

pub use crate::symbol_resolver::{
  TypeScriptFunctionParameter, TypeScriptFunctionSignature, TypeScriptObjectProperty, TypeScriptSymbol,
  TypeScriptSymbolResolver,
};
#[cfg(feature = "codegen")]
pub use crate::typescript_item_renderer::TypeScriptItemRenderer;
pub use crate::typescript_source::TypeScriptSource;
pub use crate::typescript_type_renderer::TypeScriptTypeRenderer;
