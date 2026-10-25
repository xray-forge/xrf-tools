//! The desktop app's view of a declarations tree: every extern projected into a serializable descriptor, with the
//! manifest it came from and a way to read a declaration's source back.

pub(crate) mod descriptor;
pub(crate) mod export_source_content;
pub(crate) mod exports_project;
pub(crate) mod exports_project_parser;

#[cfg(test)]
mod tests;

pub use crate::project::descriptor::{
  ExportContractDescriptor, ExportDescriptor, ExportParameterDescriptor, ExportReturnDescriptor, ExportSourceDescriptor,
};
pub use crate::project::export_source_content::ExportSourceContent;
pub use crate::project::exports_project::ExportsProject;
pub use crate::project::exports_project_parser::ExportsProjectParser;
