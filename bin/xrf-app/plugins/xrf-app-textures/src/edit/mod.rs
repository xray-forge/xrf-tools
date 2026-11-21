//! Writing, which only the editor does: the descriptor fields a person edits, where an edit lands with the stamp of
//! what is there, saving a node's two files or refusing to, saving a copy under a new name, and rebuilding a texture or
//! its bump pair from images.

pub(crate) mod commands;
mod form;
mod outcome;
mod request;
mod save;
mod save_as;

pub(crate) use form::{TextureDescriptorForm, TextureEditTargets, TextureSaveTarget};
pub(crate) use outcome::{
  TextureBuildOmissionReport, TextureBuildOutcome, TextureMakeBumpOutcome, TextureSaveAsOutcome, TextureSaveOutcome,
};
pub(crate) use request::{
  TextureDescriptorSave, TextureEncodingSave, TextureSaveAsTarget, TexturesBuildRequest, TexturesMakeBumpRequest,
  TexturesPlanSaveAsRequest, TexturesSaveAsRequest, TexturesSaveRequest,
};
pub(crate) use save::{TextureSaver, TextureWrite};
pub(crate) use save_as::{
  TextureReferenceField, TextureReferenceRename, TextureSaveAsBump, TextureSaveAsFile, TextureSaveAsPlan,
  TextureSaveAsRole, TextureSaveAsWriter,
};
