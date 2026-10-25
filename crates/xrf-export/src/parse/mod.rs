//! Reading the manifest out of a declarations tree: which sources take part, each top-level `extern(...)` call they
//! make, and the names those calls write together.

pub(crate) mod declaration;
pub(crate) mod extern_manifest_parser;
pub(crate) mod extern_source_location;
pub(crate) mod parsed_extern;
pub(crate) mod parsed_extern_manifest;

#[cfg(test)]
mod tests;

pub use crate::parse::extern_manifest_parser::ExternManifestParser;
pub use crate::parse::extern_source_location::ExternSourceLocation;
pub use crate::parse::parsed_extern::ParsedExtern;
pub use crate::parse::parsed_extern_manifest::ParsedExternManifest;
