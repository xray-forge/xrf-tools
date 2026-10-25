//! The manifest: every name the declarations publish, with the contract each one is called or read through.

pub(crate) mod extern_callable;
pub(crate) mod extern_documentation;
pub(crate) mod extern_export;
pub(crate) mod extern_manifest;
pub(crate) mod extern_parameter;
pub(crate) mod extern_value;

pub use crate::manifest::extern_callable::ExternCallable;
pub use crate::manifest::extern_documentation::ExternDocumentation;
pub use crate::manifest::extern_export::ExternExport;
pub use crate::manifest::extern_manifest::ExternManifest;
pub use crate::manifest::extern_parameter::ExternParameter;
pub use crate::manifest::extern_value::ExternValue;
