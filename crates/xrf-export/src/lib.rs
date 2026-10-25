#![doc = include_str!("../README.md")]

pub(crate) mod manifest;
pub(crate) mod parse;
pub(crate) mod project;
pub(crate) mod render;

pub use crate::manifest::{
  ExternCallable, ExternDocumentation, ExternExport, ExternManifest, ExternParameter, ExternValue,
};
pub use crate::parse::{ExternManifestParser, ExternSourceLocation, ParsedExtern, ParsedExternManifest};
pub use crate::project::{
  ExportContractDescriptor, ExportDescriptor, ExportParameterDescriptor, ExportReturnDescriptor, ExportSourceContent,
  ExportSourceDescriptor, ExportsProject, ExportsProjectParser,
};
pub use crate::render::ExternFormat;
