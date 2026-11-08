//! Writing, which only the editor does: the descriptor fields a person edits, where an edit lands with the stamp of
//! what is there, saving a node's two files or refusing to, and rebuilding a texture or its bump pair from images.

pub(crate) mod commands;
mod form;
mod outcome;
mod request;
mod save;

pub(crate) use form::{TextureDescriptorForm, TextureEditTargets, TextureSaveTarget};
pub(crate) use outcome::{TextureBuildOmissionReport, TextureBuildOutcome, TextureMakeBumpOutcome, TextureSaveOutcome};
pub(crate) use request::{
  TextureDescriptorSave, TextureEncodingSave, TexturesBuildRequest, TexturesMakeBumpRequest, TexturesSaveRequest,
};
pub(crate) use save::{TextureSaver, TextureWrite};
